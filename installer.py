import subprocess
import os
import sys
import platform

def run_command(command, cwd=None, shell=True):
    print(f"Running command: {&apos; &apos;.join(command) if isinstance(command, list) else command}")
    try:
        result = subprocess.run(
            command,
            cwd=cwd,
            shell=shell,
            check=True,
            capture_output=True,
            text=True
        )
        print("Stdout:")
        print(result.stdout)
        if result.stderr:
            print("Stderr:")
            print(result.stderr)
        return True
    except FileNotFoundError:
        print(f"Error: Command not found. Make sure the necessary tools are installed and in your PATH.")
        return False
    except subprocess.CalledProcessError as e:
        print(f"Error executing command: {e}")
        print("Stdout:")
        print(e.stdout)
        print("Stderr:")
        print(e.stderr)
        return False
    except Exception as e:
        print(f"An unexpected error occurred: {e}")
        return False

def install_dependencies_linux():
    print("Attempting to install dependencies for Linux (Kali/Ubuntu)...")
    print("You may be prompted for your sudo password.")

    commands = [
        [&apos;sudo&apos;, &apos;apt&apos;, &apos;update&apos;],
        [&apos;sudo&apos;, &apos;apt&apos;, &apos;install&apos;, &apos;-y&apos;, &apos;perl&apos;, &apos;openssl&apos;, &apos;python3&apos;, &apos;python3-pip&apos;],
        [&apos;curl&apos;, &apos;--proto&apos;, &apos;=https&apos;, &apos;--tlsv1.2&apos;, &apos;-sSf&apos;, &apos;https://sh.rustup.rs&apos;], # Rust installer script
        [&apos;sudo&apos;, &apos;cpan&apos;, &apos;App::cpanminus&apos;], # Install cpanm
        [&apos;sudo&apos;, &apos;cpanm&apos;, &apos;Mojolicious&apos;, &apos;IO::Socket::SSL&apos;, &apos;File::Slurp&apos;, &apos;JSON&apos;, &apos;Scalar::Util&apos;, &apos;Time::Piece&apos;, &apos;MIME::Base64&apos;], # Install Perl modules
        [sys.executable, &apos;-m&apos;, &apos;pip&apos;, &apos;install&apos;, &apos;-r&apos;, &apos;requirements.txt&apos;] # Install Python deps
    ]

    for command in commands:
        # Special handling for rustup script - needs to be piped to sh
        if &apos;sh.rustup.rs&apos; in &apos; &apos;.join(command):
            print("Running Rust installer script. Follow prompts if any.")
            try:
                process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                rustup_install_command = [&apos;sh&apos;]
                rustup_process = subprocess.run(
                    rustup_install_command,
                    stdin=process.stdout,
                    check=True,
                    capture_output=True,
                    text=True
                )
                process.stdout.close()
                rustup_process.wait()
                print("Rust installer output:")
                print(rustup_process.stdout)
                if rustup_process.stderr:
                    print("Rust installer errors/warnings:")
                    print(rustup_process.stderr)
                print("Rust installation finished. You may need to run &apos;source $HOME/.cargo/env&apos; or restart your terminal.")

            except FileNotFoundError:
                 print("Error: curl or sh command not found.")
                 return False
            except subprocess.CalledProcessError as e:
                print(f"Error during Rust installation: {e}")
                print("Stdout:")
                print(e.stdout)
                print("Stderr:")
                print(e.stderr)
                return False
            except Exception as e:
                print(f"An unexpected error occurred during Rust installation: {e}")
                return False

        else:
            if not run_command(command):
                print(f"Installation failed for command: {&apos; &apos;.join(command)}")
                print("Please try running this command manually to diagnose the issue.")
                return False # Stop if a command fails

    print("Linux dependency installation steps attempted. Please verify installations manually.")
    print("Remember to install Android Studio and JDK separately.")
    return True

def install_dependencies_macos():
    print("Attempting to install dependencies for macOS...")
    print("You may be prompted for your sudo password and Homebrew may ask for confirmation.")

    commands = [
        [&apos;brew&apos;, &apos;update&apos;], # Assuming Homebrew is installed
        [&apos;brew&apos;, &apos;install&apos;, &apos;perl&apos;, &apos;openssl&apos;, &apos;python@3&apos;],
        [&apos;curl&apos;, &apos;--proto&apos;, &apos;=https&apos;, &apos;--tlsv1.2&apos;, &apos;-sSf&apos;, &apos;https://sh.rustup.rs&apos;], # Rust installer script
        [&apos;sudo&apos;, &apos;cpan&apos;, &apos;App::cpanminus&apos;], # Install cpanm
        [&apos;sudo&apos;, &apos;cpanm&apos; , &apos;Mojolicious&apos;, &apos;IO::Socket::SSL&apos;, &apos;File::Slurp&apos;, &apos;JSON&apos;, &apos;Scalar::Util&apos;, &apos;Time::Piece&apos;, &apos;MIME::Base64&apos;], # Install Perl modules
        [sys.executable, &apos;-m&apos;, &apos;pip&apos;, &apos;install&apos;, &apos;-r&apos;, &apos;requirements.txt&apos;] # Install Python deps
    ]

    for command in commands:
         # Special handling for rustup script - needs to be piped to sh
        if &apos;sh.rustup.rs&apos; in &apos; &apos;.join(command):
            print("Running Rust installer script. Follow prompts if any.")
            try:
                process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                rustup_install_command = [&apos;sh&apos;]
                rustup_process = subprocess.run(
                    rustup_install_command,
                    stdin=process.stdout,
                    check=True,
                    capture_output=True,
                    text=True
                )
                process.stdout.close()
                rustup_process.wait()
                print("Rust installer output:")
                print(rustup_process.stdout)
                if rustup_process.stderr:
                    print("Rust installer errors/warnings:")
                    print(rustup_process.stderr)
                print("Rust installation finished. You may need to run &apos;source $HOME/.cargo/env&apos; or restart your terminal.")

            except FileNotFoundError:
                 print("Error: curl or sh command not found.")
                 return False
            except subprocess.CalledProcessError as e:
                print(f"Error during Rust installation: {e}")
                print("Stdout:")
                print(e.stdout)
                print("Stderr:")
                print(e.stderr)
                return False
            except Exception as e:
                print(f"An unexpected error occurred during Rust installation: {e}")
                return False
        else:
            if not run_command(command):
                print(f"Installation failed for command: {&apos; &apos;.join(command)}")
                print("Please try running this command manually to diagnose the issue.")
                return False # Stop if a command fails

    print("macOS dependency installation steps attempted. Please verify installations manually.")
    print("Remember to install Android Studio and JDK separately.")
    return True


def install_dependencies_windows():
    print("Attempting to install dependencies for Windows...")
    print("Automated installation of system dependencies (Perl, OpenSSL, Rust, Python) on Windows is complex.")
    print("Please install the following manually:")
    print("- Perl (e.g., Strawberry Perl)")
    print("- OpenSSL")
    print("- Rust (via rustup-init.exe from rustup.rs)")
    print("- Python 3 (if not already installed)")
    print("- Android Studio and JDK")
    print("\nOnce the above are installed and in your PATH, this script will attempt to install language-specific libraries.")

    # Attempt to install Perl modules via cpanm (assuming Perl and cpanm are in PATH)
    print("\nAttempting to install Perl modules...")
    if not run_command([&apos;cpanm&apos;, &apos;Mojolicious&apos;, &apos;IO::Socket::SSL&apos;, &apos;File::Slurp&apos;, &apos;JSON&apos;, &apos;Scalar::Util&apos;, &apos;Time::Piece&apos;, &apos;MIME::Base64&apos;]):
         print("Failed to install Perl modules. Ensure Perl and cpanm are installed and in PATH.")
         # return False # Decide if you want to stop here or continue

    # Attempt to install Python dependencies
    print("\nAttempting to install Python dependencies...")
    if not run_command([sys.executable, &apos;-m&apos;, &apos;pip&apos;, &apos;install&apos;, &apos;-r&apos;, &apos;requirements.txt&apos;]):
         print("Failed to install Python dependencies. Ensure Python and pip are installed and in PATH.")
         # return False # Decide if you want to stop here or continue

    print("\nWindows dependency installation steps attempted. Please verify installations manually.")
    return True


def setup_project():
    print("\nSetting up project...")
    # This could include generating certs, copying files, etc.
    # For now, the server script handles cert generation.
    # The README instructs on copying the cert.
    print("Project setup steps are detailed in the README.md file.")
    print("Please refer to the README for generating certificates and copying them.")
    # Example: Generate certs (already in server.pl, but could be here)
    # cert_gen_command = [&apos;openssl&apos;, &apos;req&apos;, &apos;-x509&apos;, &apos;-nodes&apos;, &apos;-days&apos;, &apos;365&apos;, &apos;-newkey&apos;, &apos;rsa:2048&apos;, &apos;-keyout&apos;, &apos;cross_c2/perl_c2_server/server.key&apos;, &apos;-out&apos;, &apos;cross_c2/perl_c2_server/server.crt&apos;, &apos;-subj&apos;, &apos;/CN=localhost&apos;]
    # run_command(cert_gen_command)


if __name__ == "__main__":
    print("C2 Project Installer Script")
    print("---------------------------")

    os_type = platform.system()

    if os_type == "Linux":
        install_dependencies_linux()
    elif os_type == "Darwin": # macOS
        install_dependencies_macos()
    elif os_type == "Windows":
        install_dependencies_windows()
    else:
        print(f"Unsupported operating system: {os_type}")
        print("Please install dependencies manually according to the README.")

    setup_project()

    print("\nInstaller script finished. Please review the output for any errors and complete manual steps outlined in the README.")
    print("Refer to README.md for instructions on building and running the project.")