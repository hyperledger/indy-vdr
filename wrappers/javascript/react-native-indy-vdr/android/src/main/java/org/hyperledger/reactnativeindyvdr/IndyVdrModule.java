package org.hyperledger.reactnativeindyvdr;

import androidx.annotation.NonNull;

import com.facebook.react.bridge.Promise;
import com.facebook.react.bridge.ReactApplicationContext;
import com.facebook.react.bridge.ReactContextBaseJavaModule;
import com.facebook.react.bridge.ReactMethod;
import com.facebook.react.module.annotations.ReactModule;

@ReactModule(name = IndyVdrModule.NAME)
public class IndyVdrModule extends ReactContextBaseJavaModule {
    public static final String NAME = "IndyVdr";

    public IndyVdrModule(ReactApplicationContext reactContext) {
        super(reactContext);
    }

    @Override
    @NonNull
    public String getName() {
        return NAME;
    }

    static {
        try {
            System.loadLibrary("indy-vdr");
        } catch (Exception ignored) {
        }
    }
}
