# Cross-Platform Remote Input Logging and Command-Response System (C2)

This project implements a basic cross-platform Command and Control (C2) system designed for personal lab testing and educational purposes in isolated environments. It consists of a Perl server, a Rust client for Windows, and a Java client for Android, communicating over encrypted HTTPS. A simple web panel is included for interaction.

MIT License

Copyright (c) 2025 [Your Name]

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the “Software”), to deal
in the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF
CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE
OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

⚠️ This software is intended strictly for educational, ethical hacking, or research purposes
in controlled environments. It must **not** be used to violate privacy, law, or the rights
of others. The authors are not responsible for any misuse.

## Folder Structure

```
cross_c2/
├── AndroidClient/          # Android client source code (Java)
├── windows_client/         # Windows client source code (Rust)
├── perl_c2_server/         # Perl C2 server and web panel code
├── builder.py              # Python script to build clients
├── installer.py            # Python script to assist with dependency installation
├── requirements.txt        # Python dependencies for installer/builder
└── README.md               # This file
```

## How the System Works

This C2 system operates on a client-server model using HTTPS for encrypted communication:

1.  **Server (Perl):** The central component. It listens for incoming HTTPS connections from clients on port 443. It serves a web panel on HTTP port 444 for administrative interaction. The server manages connected clients, queues commands for them, receives and logs keylogs and command outputs, and handles file uploads/downloads initiated via the web panel.
2.  **Clients (Rust for Windows, Java for Android):** These run on target machines. They establish persistent HTTPS connections to the server.
    *   **Input Logging:** The Windows client uses `device_query` to capture keyboard input. The Android client uses an `AccessibilityService` to detect text input changes.
    *   **Command Polling:** Clients periodically send GET requests to the server to check for pending commands.
    *   **Command Execution:** Upon receiving commands, clients execute them (either via the system shell for Windows, or specific Android actions/placeholders for Android) and send the results back to the server via POST requests.
    *   **File Transfer:** Clients can upload files to the server or download files from the server based on commands received from the admin panel. File content is Base64 encoded for transport within JSON payloads.
3.  **Web Panel (Perl/Mojolicious):** A simple web interface hosted by the server. It allows an administrator to view connected clients, see live keylog and command output feeds, send commands to specific clients, and manage file uploads/downloads.

Communication between clients and the server uses JSON payloads over HTTPS POST (for sending data) and GET (for fetching commands).

## Prerequisites

To set up and run this project, you will need:

*   **Perl:** With necessary modules (`Mojolicious`, `IO::Socket::SSL`, `File::Slurp`, `JSON`, `Scalar::Util`, `Time::Piece`, `MIME::Base64`). `cpanm` is recommended for module installation.
*   **OpenSSL:** For generating the self-signed certificate.
*   **Rust:** With Cargo (Rust's package manager).
*   **Android Studio:** With Android SDK and Gradle configured, for building the Android client.
*   **Java Development Kit (JDK):** Required for Android development.
*   **Python 3:** With `pip`, for running the `builder.py` and `installer.py` scripts.

### Using the Installer Script (`installer.py`)

A helper script `installer.py` is provided to assist with installing some of the prerequisites. **Note:** This script attempts to run system commands and may require manual intervention, especially on Windows or if permissions are needed. It does not install Android Studio or JDK, which must be done manually.

1.  **Navigate to the project root directory:**
    ```bash
    cd cross_c2/
    ```
2.  **Run the installer script:**
    ```bash
    python installer.py
    ```
3.  **Review the output:** The script will print the commands it is attempting to run and their results. If any command fails, you may need to run it manually or troubleshoot the issue based on your operating system and environment.
4.  **Manually install remaining prerequisites:** Ensure Perl, OpenSSL, Rust, Python 3, Android Studio, and JDK are fully installed and configured on your system as required. Refer to the OS-specific instructions below and official documentation for these tools.

### OS-Specific Dependency Installation (Manual Steps)

If the installer script does not work or you prefer to install dependencies manually, follow these steps based on your operating system.

*   **Kali Linux / Ubuntu:**
    ```bash
    sudo apt update
    sudo apt install perl openssl python3 python3-pip
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # Install Rust and Cargo. Follow prompts. You may need to source $HOME/.cargo/env afterwards.
    sudo cpan App::cpanminus # Install cpanm
    sudo cpanm Mojolicious IO::Socket::SSL File::Slurp JSON Scalar::Util Time::Piece MIME::Base64 # Install Perl modules
    pip3 install -r requirements.txt # Install Python dependencies
    ```

*   **macOS:**
    ```bash
    brew update # Assuming Homebrew is installed. Install from https://brew.sh/ if needed.
    brew install perl openssl python@3
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh # Install Rust and Cargo. Follow prompts. You may need to source $HOME/.cargo/env afterwards.
    sudo cpan App::cpanminus # Install cpanm
    sudo cpanm Mojolicious IO::Socket::SSL File::Slurp JSON Scalar::Util Time::Piece MIME::Base64 # Install Perl modules
    pip3 install -r requirements.txt # Install Python dependencies
    ```

*   **Windows:**
    Refer to the official documentation for installing:
    *   Perl (e.g., Strawberry Perl: [https://strawberryperl.com/](https://strawberryperl.com/))
    *   OpenSSL (often included with Perl distributions or available separately)
    *   Rust (via `rustup-init.exe` from [https://rustup.rs/](https://rustup.rs/))
    *   Python 3 (from [https://www.python.org/](https://www.python.org/))
    *   Android Studio and JDK (from [https://developer.android.com/studio](https://developer.android.com/studio))
    Once Perl and Python are installed and in your PATH, you can install language-specific modules:
    ```bash
    cpanm Mojolicious IO::Socket::SSL File::Slurp JSON Scalar::Util Time::Piece MIME::Base64 # Install Perl modules
    pip install -r requirements.txt # Install Python dependencies
    ```

## Server Setup and Running (Perl)

1.  **Navigate to the server directory:**
    ```bash
    cd cross_c2/perl_c2_server
    ```

2.  **Generate Self-Signed Certificate:**
    The server requires an SSL certificate for HTTPS. The `server.pl` script will attempt to generate a self-signed certificate (`server.crt` and `server.key`) if they don't exist when the server runs. This requires OpenSSL to be in your system's PATH.
    ```bash
    # This command is executed by server.pl if certs are missing
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 -keyout server.key -out server.crt -subj '/CN=localhost'
    ```
    Ensure `server.crt` and `server.key` are in the `perl_c2_server` directory.

3.  **Run the Server:**
    The server runs as a daemon listening on HTTPS port 443 (for clients) and HTTP port 444 (for the web panel, optional for development).
    ```bash
    perl server.pl daemon
    ```
    The server will log incoming data to `keylog.txt` and `cmd_output.txt` in the `perl_c2_server` directory. Uploaded files will be stored in the `uploads` directory.

4.  **Access the Web Panel:**
    Open your web browser and go to `http://localhost:444` (or `https://localhost:443` if you configure your browser to trust the self-signed cert).
    The default login is `admin` / `password123`. **Change this immediately for any testing beyond basic local setup.**

## Building Clients with `builder.py`

The `builder.py` script automates the build process for both the Windows and Android clients.

1.  **Navigate to the project root directory:**
    ```bash
    cd cross_c2/
    ```

2.  **Run the builder script:**
    ```bash
    python builder.py
    ```
    This script will execute the necessary build commands for both clients.

3.  **Build Output Locations:**
    *   **Windows Client (EXE):** The compiled executable will be found at `cross_c2/windows_client/target/release/windows_client.exe`.
    *   **Android Client (APK):** The compiled APK will typically be found at `cross_c2/AndroidClient/app/build/outputs/apk/release/app-release.apk`. The exact path might be slightly different depending on your Gradle configuration and build variants, so check the script's output for details.

## Windows Client Setup and Running (Rust)

1.  **Navigate to the Windows client directory:**
    ```bash
    cd cross_c2/windows_client
    ```

2.  **Change Server IP Address:**
    Edit the `cross_c2/windows_client/src/main.rs` file. Find the line defining `SERVER_URL`:
    ```rust
    const SERVER_URL: &'static str = "https://localhost:443";
    ```
    Replace `localhost` with the IP address or hostname of your C2 server.

3.  **Copy Server Certificate:**
    Copy the generated `server.crt` file from `cross_c2/perl_c2_server/` to the `cross_c2/windows_client/` directory. The Rust client is configured to look for it there.

4.  **Build the Client:**
    Use `cargo build --release` (or the `builder.py` script from the project root).
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/windows_client.exe`.

5.  **Run the Client:**
    Execute the compiled binary.
    ```bash
    ./target/release/windows_client.exe
    ```
    The client will attempt to connect to the configured `SERVER_URL`.

## Android Client Setup and Running (Java)

1.  **Navigate to the Android client directory:**
    ```bash
    cd cross_c2/AndroidClient
    ```

2.  **Open the project in Android Studio.**

3.  **Change Server IP Address:**
    Edit the `cross_c2/AndroidClient/app/src/main/java/com/example/crossc2/ApiClient.java` file. Find the line defining `SERVER_URL`:
    ```java
    private static final String SERVER_URL = "https://your_server_ip:443"; // Replace with your server IP
    ```
    Replace `your_server_ip` with the IP address or hostname of your C2 server.

4.  **Copy Server Certificate:**
    Copy the generated `server.crt` file from `cross_c2/perl_c2_server/` into your Android project's resources. A common place is `app/src/main/res/raw/server.crt`. You will need to create the `raw` directory if it doesn't exist.
    **Important:** Update the `ApiClient.java` file to correctly load the certificate from where you placed it (see the `getCertificateInputStream` method and the constructor).

5.  **Build the APK:**
    Use `./gradlew assembleRelease` (or `gradlew.bat` on Windows) from the `cross_c2/AndroidClient/` directory (or use the `builder.py` script from the project root).
    ```bash
    ./gradlew assembleRelease # On Linux/macOS
    # OR
    gradlew.bat assembleRelease # On Windows
    ```
    The release APK will be located in `app/build/outputs/apk/release/`.

6.  **Install the APK:**
    Transfer the generated `app-release.apk` file to your Android device or emulator and install it. You may need to enable installation from unknown sources.

7.  **Enable Accessibility Service:**
    On the Android device, go to `Settings > Accessibility > Installed services` (the exact path may vary by Android version) and enable the "C2AccessibilityService". This is required for the service to capture text input.

8.  **Run the Client:**
    The Accessibility Service should start automatically once enabled. It will run in the background.

## Communication Details

*   **Protocol:** HTTPS (TLS/SSL)
*   **Server Port:** 443 (default for HTTPS)
*   **Web Panel Port:** 444 (HTTP, for development convenience)
*   **Data Format:** JSON
*   **Client -> Server:** POST requests to `/client/data` with a JSON body containing `type` (`keylog`, `cmd_output`, `file_chunk`) and `data`/`filename`/`chunk`/`offset`/`total_size` fields.
*   **Server -> Client:** Clients poll the server with GET requests to `/client/commands`. The server responds with a JSON array of pending commands.
*   **Commands:** Commands are sent as JSON objects. Basic shell commands are sent as `{ "command": "your command here" }`. File transfer commands have additional fields:
    *   `upload <filepath>` (sent from admin to client): Client receives `{ "command": "upload", "filepath": "/path/to/file" }`. Client then reads the file, Base64 encodes chunks, and sends them to the server via `/client/data` with `type: "file_chunk"`.
    *   `download <filename>` (sent from admin to client): Client receives `{ "command": "download", "filename": "file.txt", "content": "base64content" }`. Client Base64 decodes the content and saves the file.
*   **File Transfer:** Files are transferred in chunks, Base64 encoded within JSON payloads sent via POST requests.

## Security Considerations (For Lab Testing ONLY)

*   **Self-Signed Certificates:** Used for simplicity in a lab environment. Browsers and clients will typically warn about these. For production, a proper CA-signed certificate is required.
*   **Simple Authentication:** The web panel uses basic username/password authentication hardcoded in the server script. This is highly insecure for production.
*   **Client Identification:** Clients are currently identified by IP address. This is unreliable and insecure. A robust C2 would use unique, persistent client IDs and potentially mutual TLS authentication.
*   **No Traffic Obfuscation (over HTTPS):** While the original request mentioned XOR, HTTPS provides encryption. Additional obfuscation layers could be added but are beyond the scope of this basic lab example.
*   **Android Accessibility Service:** Requires user permission to enable and can be detected.
*   **Windows Client Persistence:** The current Rust client does not implement persistence mechanisms (e.g., running on startup, hiding process).

## Usage via Web Panel

1.  Log in to the web panel (`http://localhost:444`).
2.  View connected clients in the "Connected Clients" section.
3.  Send commands to a selected client using the "Send Command" form. Enter the command (e.g., `whoami` for Windows, `get clipboard` for Android).
4.  View command outputs in the "Command Output History" section.
5.  View captured keylogs in the "Keylog Feed" section.
6.  To upload a file to a client, select the client, choose the file using the "Upload File to Client" form, and submit. This queues a `download` command for the client with the file content.
7.  Files uploaded *from* clients will appear in the "Uploaded Files (from Clients)" section, where you can download them.

## Comparison with Professional C2 Frameworks (Cobalt Strike, Mythic)

This project is a very basic implementation for educational and lab testing purposes and is not comparable in features, robustness, or stealth to professional C2 frameworks like Cobalt Strike or open-source alternatives like Mythic.

**Key Differences:**

*   **Features:** Professional C2s offer a vast array of post-exploitation modules, sophisticated command and control channels (HTTP, HTTPS, DNS, SMB, etc.), in-memory execution, lateral movement capabilities, and integration with other security tools. This project provides only basic keylogging, command execution, and file transfer over HTTPS.
*   **Stealth and Evasion:** Professional frameworks employ advanced techniques to evade detection by antivirus software, EDRs, and network monitoring tools (e.g., malleable C2 profiles, reflective DLL injection, obfuscation). This project has minimal stealth features; clients are easily detectable processes, and communication patterns are simple.
*   **Scalability and Management:** Professional C2s are designed to manage large numbers of implants and operators, with features for tasking, data management, and collaboration. This project is designed for a single operator and a small number of clients in a controlled environment.
*   **Flexibility:** Frameworks like Mythic are highly extensible, allowing users to develop custom agents and modules. This project requires manual code modification to add new features.
*   **Malleability:** Cobalt Strike's malleable C2 allows operators to customize network indicators to blend in with legitimate traffic. This project uses a fixed HTTPS communication pattern.

**In summary:** This project serves as a foundational example to understand the basic concepts of C2 communication, client-server interaction, and cross-platform development in a controlled lab setting. It is a starting point for learning, not a tool for real-world adversarial simulation or operations.

## Extending the System

*   Implement more sophisticated command handling on clients (e.g., dumping contacts, taking screenshots).
*   Improve client persistence and stealth on target systems (use with caution and only in authorized labs).
*   Enhance server-side features (client grouping, task scheduling, better logging).
*   Implement real-time updates on the web panel using Websockets or Server-Sent Events.
*   Improve authentication and security for the web panel and client communication.
*   Add support for other client platforms (e.g., macOS, Linux).
*   Implement a more robust client identification system.