#pragma once
#include "rust/cxx.h"

struct HookDecision;
class PolicyHooks;

/// Must run before the QGuiApplication exists (a QtWebEngine requirement).
void initialize_web_engine();
/// Hands the default web profile its interceptor and cookie filter; the profile owns them from now on.
void install_hooks(rust::Box<PolicyHooks> hooks);
/// `Chromium/140.0.7339.207`-style version of the engine, without the prefix.
rust::String chromium_version();
