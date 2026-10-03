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
        versionCode = 1
        versionName = "1.0.0"

        ndk {
            abiFilters.addAll(listOf("arm64-v8a", "x86_64"))
        }
    }

    ndkVersion = "26.3.11579264"

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
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
}

// Gradle task to build Rust Core using cargo and the installed NDK
tasks.register("buildRustCore") {
    description = "Compiles the Rust rynk_core crate for Android ABIs"
    group = "build"

    val abiTargets = mapOf(
        "arm64-v8a" to Triple("aarch64-linux-android", "aarch64-linux-android24-clang", "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER"),
        "x86_64" to Triple("x86_64-linux-android", "x86_64-linux-android24-clang", "CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER")
    )

    doLast {
        val ndkPath = System.getenv("ANDROID_NDK_HOME")
            ?: "${System.getenv("HOME")}/Android/Sdk/ndk/26.3.11579264"
        val llvmBin = "$ndkPath/toolchains/llvm/prebuilt/linux-x86_64/bin"

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
