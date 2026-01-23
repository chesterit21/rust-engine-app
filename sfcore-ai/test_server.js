/*
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
*/

const net = require('net');

const SOCKET_PATH = '/tmp/sfcore-ai.sock';

console.log(`Connecting to ${SOCKET_PATH}...`);

const client = net.createConnection(SOCKET_PATH, () => {
    console.log('Connected!');

    // Mensimulasikan role 'system' dan 'user' dalam prompt mentah
    // Karena model ini (Coder 90M) paling baik dengan instruksi langsung.
    const system_role = "You are a senior software engineer expert in Rust.";
    const user_role = "explain Vect in rust programming.";
    
    const req = {
        // Gabungkan system dan user role secara manual ke dalam prompt
        prompt: `System: ${system_role}\nUser: ${user_role}\nAssistant: \`\`\`rust\nVect<>`,
        stream: true,
        max_tokens: 2048,
        temperature: 0.1,
        stop: ["\`\`\`", "###", "User:", "System:"]
    };

    console.log('Sending prompt with simulated System Role...');
    client.write(JSON.stringify(req) + "\n");
});

client.on('data', (data) => {
    const rawOutput = data.toString();
    const lines = rawOutput.split('\n').filter(line => line.trim() !== '');
    
    for (const line of lines) {
        try {
            const json = JSON.parse(line);
            
            // Menangani berbagai format respons (OpenAI style atau format 'token' sederhana)
            if (json.token) {
                process.stdout.write(json.token);
            } else if (json.choices && json.choices[0].delta && json.choices[0].delta.content) {
                process.stdout.write(json.choices[0].delta.content);
            } else if (json.choices && json.choices[0].text) {
                process.stdout.write(json.choices[0].text);
            } else if (json.content) {
                process.stdout.write(json.content);
            }
        } catch (e) {
            // Jika parsing gagal, cetak baris mentah (bisa jadi teks streaming langsung)
            // Hilangkan deteksi error yang terlalu berisik
            if (!line.startsWith('{')) {
                process.stdout.write(line);
            }
        }
    }
});

client.on('end', () => {
    console.log('\n\n[Done] Disconnected');
});

client.on('error', (err) => {
    console.error('Connection error:', err.message);
});

