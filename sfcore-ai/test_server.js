const net = require('net');

const SOCKET_PATH = '/tmp/sfcore-ai.sock';

console.log(`Connecting to ${SOCKET_PATH}...`);

const client = net.createConnection(SOCKET_PATH, () => {
    console.log('Connected!');

    // Send request with Chat Template
    const req = {
        messages: [
            { role: "system", "content": "Kamu adalah asisten AI yang ahli bahasa pemrograman Nodejs, TypeScript dan Framework Nestjs." },
            { role: "user", "content": "Bagaimana inisialisasi Swagger di Nestjs." }
        ],
        stream: true,
        max_tokens: 1200,
        temperature: 0.7
    };

    console.log('Sending request:', JSON.stringify(req));
    client.write(JSON.stringify(req) + "\n");
});

client.on('data', (data) => {
    console.log('Received chunk:', data.toString());
});

client.on('end', () => {
    console.log('Disconnected');
});

client.on('error', (err) => {
    console.error('Connection error:', err.message);
});
