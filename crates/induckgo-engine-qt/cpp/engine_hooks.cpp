#include "engine_hooks.h"
#include "induckgo-engine-qt/src/engine_bridge.cxx.h"

#include <QFile>
#include <QGuiApplication>
#include <QIcon>
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

void install_hooks(rust::Str storage_path, rust::Str cache_path) {
    QQuickWebEngineProfile *profile = QQuickWebEngineProfile::defaultProfile();
    // The directories are set before the profile becomes persistent: making it persistent first would create
    // the engine's files (history, favicons) in Qt's default location, outside the directory burning deletes.
    profile->setPersistentStoragePath(QString::fromUtf8(storage_path.data(), static_cast<int>(storage_path.size())));
    profile->setCachePath(QString::fromUtf8(cache_path.data(), static_cast<int>(cache_path.size())));
    // The default profile of QtWebEngine Quick keeps nothing on disk until it is made persistent.
    profile->setStorageName(QStringLiteral("InDuckGo"));
    profile->setOffTheRecord(false);
    profile->setPersistentCookiesPolicy(QQuickWebEngineProfile::AllowPersistentCookies);
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

void set_window_icon(rust::Str resource_path) {
    QGuiApplication::setWindowIcon(
        QIcon(QString::fromUtf8(resource_path.data(), static_cast<int>(resource_path.size()))));
}

rust::String read_qt_resource(rust::Str path) {
    QFile file(QString::fromUtf8(path.data(), static_cast<int>(path.size())));
    if (!file.open(QIODevice::ReadOnly)) {
        return rust::String();
    }
    return rust::String(QString::fromUtf8(file.readAll()).toStdString());
}
