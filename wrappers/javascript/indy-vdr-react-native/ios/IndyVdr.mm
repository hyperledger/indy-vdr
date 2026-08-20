#import <React/RCTBridge+Private.h>
#import <jsi/jsi.h>
#import <React/RCTUtils.h>
#import <ReactCommon/RCTTurboModule.h>
#import <ReactCommon/RCTInteropTurboModule.h>
#import <ReactCommon/RCTTurboModuleWithJSIBindings.h>

#import "turboModuleUtility.h"
#import "IndyVdr.h"

using namespace facebook;

@interface IndyVdr () <RCTTurboModule, RCTTurboModuleWithJSIBindings>
@end

@implementation IndyVdr

RCT_EXPORT_MODULE()

// Expose this module as a TurboModule so React Native invokes
// installJSIBindingsWithRuntime:callInvoker: below when the module is created.
// This is the only supported way to install JSI bindings under the New
// Architecture / bridgeless mode, where [RCTBridge currentBridge] is nil and
// the legacy bridge is compiled out (RCT_REMOVE_LEGACY_ARCH=1).
//
// ObjCInteropTurboModule keeps the existing RCT_EXPORT methods (e.g. `install`)
// dispatchable without requiring a codegen spec, so this remains a drop-in
// change that also works on the old architecture.
- (std::shared_ptr<react::TurboModule>)getTurboModule:(const react::ObjCTurboModule::InitParams &)params
{
    return std::make_shared<react::ObjCInteropTurboModule>(params);
}

- (void)installJSIBindingsWithRuntime:(jsi::Runtime &)runtime
                          callInvoker:(const std::shared_ptr<react::CallInvoker> &)callInvoker
{
    indyVdrTurboModuleUtility::registerTurboModule(runtime, callInvoker);
}

RCT_EXPORT_BLOCKING_SYNCHRONOUS_METHOD(install)
{
    // New Architecture: the JSI bindings are installed via
    // installJSIBindingsWithRuntime:callInvoker: when this TurboModule is
    // created, so there is nothing to do here.
    //
    // Old Architecture (legacy bridge present): install using the bridge
    // runtime so this keeps working for non-bridgeless apps.
    RCTBridge* bridge = [RCTBridge currentBridge];
    RCTCxxBridge* cxxBridge = (RCTCxxBridge*)bridge;
    if (cxxBridge != nil) {
        jsi::Runtime* jsiRuntime = (jsi::Runtime*) cxxBridge.runtime;
        if (jsiRuntime != nil) {
            indyVdrTurboModuleUtility::registerTurboModule(*jsiRuntime, bridge.jsCallInvoker);
        }
    }
    return @true;
}

@end
