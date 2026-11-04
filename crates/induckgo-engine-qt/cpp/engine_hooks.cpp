#include "engine_hooks.h"
#include "induckgo-engine-qt/src/engine_bridge.cxx.h"

#include <QQuickWebEngineProfile>
#include <QWebEngineCookieStore>
#include <QWebEngineUrlRequestInfo>
#include <QWebEngineUrlRequestInterceptor>
#include <QtWebEngineCore/qtwebenginecoreglobal.h>
#include <QtWebEngineQuick>

namespace {

/// Called by Chromium on its IO thread for every request; the decision itself is made in Rust.
class DecidingInterceptor : public QWebEngineUrlRequestInterceptor {
public:
    explicit DecidingInterceptor(rust::Rc<PolicyHooks> hooks) : hooks_(std::move(hooks)) {}

    void interceptRequest(QWebEngineUrlRequestInfo &info) override {
        const QByteArray url = info.requestUrl().toEncoded();
        const QByteArray page = info.firstPartyUrl().toEncoded();
        const HookDecision decision = hooks_->decide_request(
            rust::Str(url.constData(), static_cast<size_t>(url.size())),
            rust::Str(page.constData(), static_cast<size_t>(page.size())),
            static_cast<std::int32_t>(info.resourceType()), next_request_id());
        if (decision.action == 1) {
            info.block(true);
        } else if (decision.action == 2) {
            info.redirect(QUrl(QString::fromUtf8(decision.redirect_to.data(), static_cast<int>(decision.redirect_to.size()))));
        }
    }

private:
    static std::uint64_t next_request_id() {
        static std::atomic<std::uint64_t> counter{0};
        return ++counter;
    }
    rust::Rc<PolicyHooks> hooks_;
};

} // namespace
