import java.util.Properties

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "org.rynk.keyboard"
    compileSdk = 35

    defaultConfig {
        applicationId = "org.rynk.keyboard"
        minSdk = 24
        targetSdk = 35
        versionCode = 2
        versionName = "1.1.0"

        ndk {
            abiFilters.addAll(listOf("arm64-v8a", "armeabi-v7a", "x86_64"))
        }
    }

    ndkVersion = "26.3.11579264"

    signingConfigs {
        create("release") {
            val localProps = Properties()
            val localPropsFile = rootProject.file("local.properties")
            if (localPropsFile.exists()) {
                localProps.load(localPropsFile.inputStream())
            }

            val keystorePath = System.getenv("RYNK_KEYSTORE_PATH")
                ?: System.getenv("RYNK_RELEASE_STORE_FILE")
                ?: localProps.getProperty("rynk.release.storeFile")
            val keystorePass = System.getenv("RYNK_KEYSTORE_PASSWORD")
                ?: System.getenv("RYNK_RELEASE_STORE_PASSWORD")
                ?: localProps.getProperty("rynk.release.storePassword")
            val keyAliasStr = System.getenv("RYNK_KEY_ALIAS")
                ?: System.getenv("RYNK_RELEASE_KEY_ALIAS")
                ?: localProps.getProperty("rynk.release.keyAlias")
            val keyPassStr = System.getenv("RYNK_KEY_PASSWORD")
                ?: System.getenv("RYNK_RELEASE_KEY_PASSWORD")
                ?: localProps.getProperty("rynk.release.keyPassword")

            // The keystore and its passwords never live in the repository: environment variables
            // or local.properties (git-ignored); without them the release build is unsigned
            if (!keystorePath.isNullOrBlank() && file(keystorePath).exists() &&
                !keystorePass.isNullOrBlank() && !keyAliasStr.isNullOrBlank() && !keyPassStr.isNullOrBlank()) {
                storeFile = file(keystorePath)
                storePassword = keystorePass
                keyAlias = keyAliasStr
                keyPassword = keyPassStr
            }
            enableV1Signing = true
            enableV2Signing = true
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            val releaseSigning = signingConfigs.getByName("release")
            if (releaseSigning.storeFile != null && releaseSigning.storeFile!!.exists()) {
                signingConfig = releaseSigning
            } else {
                signingConfig = null
            }
        }
        debug {
            isDebuggable = true
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_21
        targetCompatibility = JavaVersion.VERSION_21
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }

    lint {
        abortOnError = true
        checkReleaseBuilds = true
        disable.addAll(listOf("MissingTranslation", "ClickableViewAccessibility"))
    }
}

kotlin {
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_21)
    }
}

dependencies {
    implementation("androidx.core:core-ktx:1.15.0")
    implementation("androidx.appcompat:appcompat:1.7.0")
    implementation("com.google.android.material:material:1.12.0")
    implementation("androidx.preference:preference-ktx:1.2.1")

    testImplementation("junit:junit:4.13.2")
}

// Gradle task to build Rust Core using cargo and the installed NDK
tasks.register("buildRustCore") {
    description = "Compiles the Rust rynk_core crate for Android ABIs"
    group = "build"

    val abiTargets = mapOf(
        "arm64-v8a" to Triple("aarch64-linux-android", "aarch64-linux-android24-clang", "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER"),
        "armeabi-v7a" to Triple("armv7-linux-androideabi", "armv7a-linux-androideabi24-clang", "CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER"),
        "x86_64" to Triple("x86_64-linux-android", "x86_64-linux-android24-clang", "CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER")
    )

    doLast {
        val ndkPath = System.getenv("ANDROID_NDK_HOME")
            ?: System.getenv("ANDROID_NDK_ROOT")
            ?: "${System.getenv("ANDROID_HOME") ?: "${System.getenv("HOME")}/Android/Sdk"}/ndk/26.3.11579264"
        val prebuiltDir = file("$ndkPath/toolchains/llvm/prebuilt").listFiles()?.firstOrNull { it.isDirectory }
        val llvmBin = prebuiltDir?.resolve("bin")?.absolutePath
            ?: "$ndkPath/toolchains/llvm/prebuilt/linux-x86_64/bin"

        abiTargets.forEach { (abi, triple) ->
            val (rustTarget, clangBinary, linkerEnv) = triple
            val clangPath = "$llvmBin/$clangBinary"
            val arPath = "$llvmBin/llvm-ar"
            val jniOutputDir = file("src/main/jniLibs/$abi")
            jniOutputDir.mkdirs()

            println("Building Rust rynk_core for $abi ($rustTarget)...")

            val pb = ProcessBuilder("cargo", "build", "--release", "--target", rustTarget, "-p", "rynk_core")
            pb.directory(rootDir)
            pb.environment()[linkerEnv] = clangPath
            pb.environment()["CC_$rustTarget"] = clangPath
            pb.environment()["AR_$rustTarget"] = arPath
            val normalizedTarget = rustTarget.replace('-', '_')
            pb.environment()["CC_$normalizedTarget"] = clangPath
            pb.environment()["AR_$normalizedTarget"] = arPath
            val exitCode = pb.inheritIO().start().waitFor()
            if (exitCode != 0) {
                throw GradleException("Failed to build Rust library for $rustTarget with exit code $exitCode")
            }

            val builtLib = file("${rootDir}/target/$rustTarget/release/librynk_core.so")
            val targetLib = file("$jniOutputDir/librynk_core.so")
            builtLib.copyTo(targetLib, overwrite = true)
            println("Successfully placed librynk_core.so at $targetLib")
        }
    }
}

tasks.named("preBuild") {
    dependsOn("buildRustCore")
}
