# ProGuard / R8 Rules for Rynk Keyboard

# Keep all native methods and JNI bridge
-keepclassmembers class * {
    native <methods>;
}

-keep class org.rynk.keyboard.NativeBridge {
    public static *;
    *;
}

# Keep InputMethodService and View components referenced by Android system / XML
-keep class org.rynk.keyboard.RynkInputMethodService {
    <init>(...);
    *;
}

-keep class org.rynk.keyboard.RynkKeyboardView {
    public <init>(android.content.Context);
    public <init>(android.content.Context, android.util.AttributeSet);
    public <init>(android.content.Context, android.util.AttributeSet, int);
    *;
}

# Keep Activities
-keep class org.rynk.keyboard.SettingsActivity {
    <init>(...);
    *;
}

-keep class org.rynk.keyboard.SetupWizardActivity {
    <init>(...);
    *;
}

# Keep model classes
-keep class org.rynk.keyboard.UserWord { *; }
-keep class org.rynk.keyboard.UserDictionaryManager { *; }
-keep class org.rynk.keyboard.HapticManager { *; }
-keep class org.rynk.keyboard.SvgIcons { *; }

# Kotlin Reflection & Coroutines (if any)
-dontwarn kotlin.**
