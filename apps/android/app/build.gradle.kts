plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
}

android {
    namespace = "com.bizsuite.devicebridge"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.bizsuite.devicebridge"
        minSdk = 26
        targetSdk = 35
        versionCode = 3
        versionName = "0.1.2"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

kotlin {
    jvmToolchain(17)
}

dependencies {
    implementation("androidx.core:core-ktx:1.15.0")
    implementation("androidx.appcompat:appcompat:1.7.0")
}
