#pragma once
#include "rust/cxx.h"

/// Must run before the QGuiApplication exists (a QtWebEngine requirement).
void initialize_web_engine();
/// Hands the default web profile its request interceptor and cookie filter.
void install_hooks();
/// Version of the Chromium inside QtWebEngine, such as `140.0.7339.207`.
rust::String chromium_version();
