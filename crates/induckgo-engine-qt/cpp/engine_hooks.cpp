#include "engine_hooks.h"
#include "induckgo-engine-qt/src/engine_bridge.cxx.h"

#include <QQuickWebEngineProfile>
#include <QWebEngineCookieStore>
#include <QWebEngineUrlRequestInfo>
#include <QWebEngineUrlRequestInterceptor>
#include <QtWebEngineCore/qtwebenginecoreglobal.h>
#include <QtWebEngineQuick>
#include <atomic>

namespace {

/// Called by Chromium on its IO thread for every request; the decision is made in Rust.
class DecidingInterceptor : public QWebEngineUrlRequestInterceptor {
public:
    void interceptRequest(QWebEngineUrlRequestInfo &info) override {
        const QByteArray url = info.requestUrl().toEncoded();
        const QByteArray page = info.firstPartyUrl().toEncoded();
        const HookDecision decision = decide_request(
            rust::Str(url.constData(), static_cast<std::size_t>(url.size())),
            rust::Str(page.constData(), static_cast<std::size_t>(page.size())),
            static_cast<std::int32_t>(info.resourceType()), ++request_counter_);
        if (decision.action == 1) {
            info.block(true);
        } else if (decision.action == 2) {
            info.redirect(QUrl(QString::fromUtf8(decision.redirect_to.data(), static_cast<int>(decision.redirect_to.size()))));
        }
    }

private:
    std::atomic<std::uint64_t> request_counter_{0};
};

} // namespace

void initialize_web_engine() {
    QtWebEngineQuick::initialize();
}

void install_hooks() {
    QQuickWebEngineProfile *profile = QQuickWebEngineProfile::defaultProfile();
    profile->setUrlRequestInterceptor(new DecidingInterceptor);
    profile->cookieStore()->setCookieFilter([](const QWebEngineCookieStore::FilterRequest &request) {
        const QByteArray domain = request.origin.host().toUtf8();
        const QByteArray page = request.firstPartyUrl.host().toUtf8();
        return allow_cookie(rust::Str(domain.constData(), static_cast<std::size_t>(domain.size())),
                            rust::Str(page.constData(), static_cast<std::size_t>(page.size())), request.thirdParty);
    });
}

rust::String chromium_version() {
    return rust::String(qWebEngineChromiumVersion());
}
