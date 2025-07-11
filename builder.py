import subprocess
import os
import sys

def build_windows_client():
    print("Building Windows client...")
    windows_client_dir = os.path.join(&apos;.&apos;, &apos;windows_client&apos;)
    if not os.path.exists(windows_client_dir):
        print(f"Error: Windows client directory not found at {windows_client_dir}")
        return

    try:
        # Run cargo build --release
        result = subprocess.run(
            [&apos;cargo&apos;, &apos;build&apos;, &apos;--release&apos;],
            cwd=windows_client_dir,
            check=True,
            capture_output=True,
            text=True
        )
        print("Cargo build output:")
        print(result.stdout)
        if result.stderr:
            print("Cargo build errors/warnings:")
            print(result.stderr)

        exe_path = os.path.join(windows_client_dir, &apos;target&apos;, &apos;release&apos;, &apos;windows_client.exe&apos;)
        if os.path.exists(exe_path):
            print(f"Windows client built successfully: {exe_path}")
        else:
            print("Error: Windows client executable not found after build.")

    except FileNotFoundError:
        print("Error: cargo command not found. Is Rust installed and in your PATH?")
    except subprocess.CalledProcessError as e:
        print(f"Error building Windows client: {e}")
        print("Stdout:")
        print(e.stdout)
        print("Stderr:")
        print(e.stderr)
    except Exception as e:
        print(f"An unexpected error occurred during Windows client build: {e}")

def build_android_client():
    print("Building Android client...")
    android_client_dir = os.path.join(&apos;.&apos;, &apos;AndroidClient&apos;)
    if not os.path.exists(android_client_dir):
        print(f"Error: Android client directory not found at {android_client_dir}")
        return

    # Determine the correct gradlew command based on OS
    if sys.platform == "win32":
        gradlew_command = [&apos;gradlew.bat&apos;, &apos;assembleRelease&apos;]
    else:
        gradlew_command = [&apos;./gradlew&apos;, &apos;assembleRelease&apos;]

    try:
        # Run gradlew assembleRelease
        result = subprocess.run(
            gradlew_command,
            cwd=android_client_dir,
            check=True,
            capture_output=True,
            text=True
        )
        print("Gradle build output:")
        print(result.stdout)
        if result.stderr:
            print("Gradle build errors/warnings:")
            print(result.stderr)

        # Expected APK path (may vary slightly based on build variants)
        apk_path_pattern = os.path.join(android_client_dir, &apos;app&apos;, &apos;build&apos;, &apos;outputs&apos;, &apos;apk&apos;, &apos;release&apos;, &apos;app-release.apk&apos;)
        # Note: Finding the exact APK path might require parsing the build output
        # For simplicity, we&apos;ll just print the expected location.
        print(f"Android client build finished. Expected APK location: {apk_path_pattern}")
        print("Please check the build output above for the exact path and any errors.")


    except FileNotFoundError:
        print("Error: gradlew command not found. Make sure you are in the AndroidClient directory and have run &apos;gradle wrapper&apos; or have a gradlew script.")
        print("Also ensure Java and Android SDK are installed and configured.")
    except subprocess.CalledProcessError as e:
        print(f"Error building Android client: {e}")
        print("Stdout:")
        print(e.stdout)
        print("Stderr:")
        print(e.stderr)
    except Exception as e:
        print(f"An unexpected error occurred during Android client build: {e}")


if __name__ == "__main__":
    print("C2 Builder Script")
    print("-----------------")

    build_windows_client()
    print("\n") # Add a newline for separation
    build_android_client()

    print("\nBuild process finished.")