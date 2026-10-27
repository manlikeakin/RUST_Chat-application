
Add-Type -TypeDefinition @"
using System;
using System.IO;
using System.Net.Sockets;
using System.Threading;

public class InteractiveChatClient
{
    private TcpClient client;
    private StreamReader reader;
    private StreamWriter writer;
    private Thread receiveThread;

    public void Connect(string host, int port, string username)
    {
        client = new TcpClient(host, port);

        NetworkStream stream = client.GetStream();
        reader = new StreamReader(stream);
        writer = new StreamWriter(stream);
        writer.AutoFlush = true;

        // Send the username first, before any chat messages.
        writer.WriteLine(username);

        receiveThread = new Thread(ReceiveMessages);
        receiveThread.IsBackground = true;
        receiveThread.Start();

        Console.WriteLine("Connected as " + username + "!");
        Console.WriteLine("Type messages and press Enter.");
        Console.WriteLine("Type /quit to leave the chat.");
    }

    private void ReceiveMessages()
    {
        try
        {
            string message;

            while ((message = reader.ReadLine()) != null)
            {
                Console.WriteLine();
                Console.WriteLine(message);
                Console.Write("> ");
            }
        }
        catch
        {
            // The connection may close when the client exits.
        }
    }

    public void Send(string message)
    {
        writer.WriteLine(message);
    }

    public void Disconnect()
    {
        if (client != null)
        {
            client.Close();
        }
    }
}
"@

# Ask the user for a username.
$username = Read-Host "Enter your username"

if ([string]::IsNullOrWhiteSpace($username)) {
    Write-Host "Username cannot be empty."
    return
}

# Connect to the Rust chat server.
$chat = [InteractiveChatClient]::new()

try {
    $chat.Connect("127.0.0.1", 7878, $username)

    while ($true) {
        $message = Read-Host "> "

        if ($message -eq "/quit") {
            break
        }

        if (-not [string]::IsNullOrWhiteSpace($message)) {
            $chat.Send($message)
        }
    }
}
catch {
    Write-Host "Could not connect to the chat server: $($_.Exception.Message)"
}
finally {
    $chat.Disconnect()
    Write-Host "Disconnected from chat."
}
