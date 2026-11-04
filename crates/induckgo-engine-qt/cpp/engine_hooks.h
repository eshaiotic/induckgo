#pragma once
#include "rust/cxx.h"

/// Must run before the QGuiApplication exists (a QtWebEngine requirement).
void initialize_web_engine();
/// Points the default web profile at its own directories and hands it the request interceptor and cookie
/// filter. Everything the engine stores goes under `storage_path` and `cache_path`, nowhere else.
void install_hooks(rust::Str storage_path, rust::Str cache_path);
/// Version of the Chromium inside QtWebEngine, such as `140.0.7339.207`.
rust::String chromium_version();
