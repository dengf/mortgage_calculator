#!/usr/bin/env bash
# Builds a properly release-signed Android APK + AAB for mortgage-ui.
#
# `dx bundle --platform android --release` compiles the Rust side in
# release mode and generates/syncs the wrapping Gradle project, but its own
# packaging step always produces a *debug*-signed APK — the generated
# build.gradle.kts has no `signingConfig` wired for the release build type,
# since dx can't assume a keystore exists. This script patches that in
# afterwards and runs Gradle's real release tasks directly.
#
# Required environment variables:
#   JAVA_HOME               - a JDK Gradle supports (17-21; NOT 26, which
#                              this project's Gradle version can't read
#                              class files from — see README)
#   ANDROID_HOME             - Android SDK root
#   ANDROID_NDK_HOME         - NDK root (needed by the Rust cross-compile)
#   ANDROID_KEYSTORE_PATH    - path to the release .keystore file
#   ANDROID_KEYSTORE_PASSWORD
#   ANDROID_KEY_ALIAS
#   ANDROID_KEY_PASSWORD
#
# Outputs (copied to crates/mortgage-ui/dist/):
#   mortgage-ui-release.apk
#   mortgage-ui-release.aab

set -euo pipefail

for var in JAVA_HOME ANDROID_HOME ANDROID_NDK_HOME ANDROID_KEYSTORE_PATH ANDROID_KEYSTORE_PASSWORD ANDROID_KEY_ALIAS ANDROID_KEY_PASSWORD; do
  if [ -z "${!var:-}" ]; then
    echo "error: $var is not set" >&2
    exit 1
  fi
done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
WORKSPACE_ROOT="$(cd "$CRATE_DIR/../.." && pwd)"
GRADLE_PROJECT="$WORKSPACE_ROOT/target/dx/mortgage-ui/release/android/app"
GRADLE_APP_MODULE="$GRADLE_PROJECT/app"

echo "==> Building release .so and syncing the Android Gradle project via dx"
export PATH="$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$ANDROID_HOME/emulator:$PATH"
(cd "$CRATE_DIR" && dx bundle --platform android --package-types apk --release)

BUILD_GRADLE="$GRADLE_APP_MODULE/build.gradle.kts"
if [ ! -f "$BUILD_GRADLE" ]; then
  echo "error: expected generated Gradle file not found at $BUILD_GRADLE" >&2
  echo "dx's generated project layout may have changed; update this script." >&2
  exit 1
fi

if ! grep -q "MORTGAGE_UI_RELEASE_SIGNING" "$BUILD_GRADLE"; then
  echo "==> Patching in a release signingConfig"
  # Insert immediately after the `android {` opening line.
  python3 - "$BUILD_GRADLE" <<'PYEOF'
import sys

path = sys.argv[1]
with open(path) as f:
    content = f.read()

signing_block = '''
    // MORTGAGE_UI_RELEASE_SIGNING - injected by build-android-release.sh
    signingConfigs {
        create("release") {
            storeFile = file(System.getenv("ANDROID_KEYSTORE_PATH")!!)
            storePassword = System.getenv("ANDROID_KEYSTORE_PASSWORD")
            keyAlias = System.getenv("ANDROID_KEY_ALIAS")
            keyPassword = System.getenv("ANDROID_KEY_PASSWORD")
        }
    }
'''

content = content.replace("android {\n", "android {\n" + signing_block, 1)
content = content.replace(
    'getByName("release") {\n',
    'getByName("release") {\n            signingConfig = signingConfigs.getByName("release")\n',
    1,
)

with open(path, "w") as f:
    f.write(content)
PYEOF
fi

echo "==> Running Gradle release build (assembleRelease + bundleRelease)"
(
  cd "$GRADLE_PROJECT"
  ./gradlew assembleRelease bundleRelease
)

APK_OUT=$(find "$GRADLE_APP_MODULE" -path "*/outputs/apk/release/*.apk" | head -1)
AAB_OUT=$(find "$GRADLE_APP_MODULE" -path "*/outputs/bundle/release/*.aab" | head -1)

if [ -z "$APK_OUT" ] || [ -z "$AAB_OUT" ]; then
  echo "error: expected release APK/AAB not found under $GRADLE_APP_MODULE/*/outputs" >&2
  exit 1
fi

mkdir -p "$CRATE_DIR/dist"
cp "$APK_OUT" "$CRATE_DIR/dist/mortgage-ui-release.apk"
cp "$AAB_OUT" "$CRATE_DIR/dist/mortgage-ui-release.aab"

echo "==> Verifying APK signature"
APKSIGNER=$(find "$ANDROID_HOME/build-tools" -name apksigner | head -1)
"$APKSIGNER" verify --print-certs "$CRATE_DIR/dist/mortgage-ui-release.apk"

echo "==> Done:"
echo "    $CRATE_DIR/dist/mortgage-ui-release.apk"
echo "    $CRATE_DIR/dist/mortgage-ui-release.aab"
