#!/usr/bin/env node
/**
 * sse_client.js
 * Client untuk mengetest HTTP Server-Sent Events (SSE) 
 */
const http = require('http');
const readline = require('readline');

async function sseChat(messages, options = {}) {
    const {
        stream = true,
        maxTokens = 2048,
        host = '127.0.0.1',
        port = 8080 // Default port for HTTP in server_config.toml
    } = options;

    const postData = JSON.stringify({
        messages,
        stream,
        max_tokens: maxTokens
    });

    const reqOptions = {
        hostname: host,
        port: port,
        path: '/v1/inference', // Updated path
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(postData),
            'Accept': 'text/event-stream',
            'Authorization': 'Bearer sk-sfcore-1234567890abcdef', // Use API Key from config
            'x-client-app': 'MyApp-v1.0' // Use allowed client name
        }
    };

    return new Promise((resolve, reject) => {
        const req = http.request(reqOptions, (res) => {
            let fullOutput = '';
            
            res.on('data', (chunk) => {
                const lines = chunk.toString().split('\n');
                for (let line of lines) {
                    if (line.startsWith('data: ')) {
                        const dataStr = line.slice(6).trim();
                        if (dataStr === '[DONE]') {
                            console.log('\n\n[SSE Status] Streaming Finished');
                            return;
                        }

                        try {
                            const json = JSON.parse(dataStr);
                            if (json.token) {
                                process.stdout.write(json.token);
                                fullOutput += json.token;
                            }
                            if (json.done) {
                                console.log('\n\n[Metrics]', json.metrics);
                                resolve(fullOutput);
                            }
                            if (json.error) {
                                console.error('\n[Error]', json.error);
                                reject(new Error(json.error));
                            }
                        } catch (e) {
                            // Mungkin baris parsial, abaikan atau handle buffer
                        }
                    }
                }
            });

            res.on('end', () => {
                resolve(fullOutput);
            });
        });

        req.on('error', (e) => {
            reject(e);
        });

        req.write(postData);
        req.end();
    });
}

/* =========================
   CLI INPUT DARI TERMINAL
   ========================= */

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

const systemPrompt = `Kamu adalah AI Expert di bidang pencarian pola atau pattern prediksi angka, mempunyai kemampuan untuk memprediksi angka berdasarkan pola atau pattern yang diberikan.
Nama Kamu adalah SFCore AI, kamu di developmen oleh SFCore Team.
 
### RULES & TASK.
*- Teliti dan perhitungkan dengan cermat [KONTEKS] yang di berikan sebelum memberikan jawaban.
*- Periode tertinggi adalah periode terbaru, dan data ResultLog nya adalah data terbaru.
*- Tugas kamu memberikan 10 Angka berupa 2 Digit di bagian depan yang kemungkinan nya tidak akan keluar.
*- 2 Digit itu di mulai dari 00 sampai dengan 99. Jadi tidak ada 2 Digit yang lebih besar dari 99 dan tidak ada 2 Digit yang lebih kecil dari 00.
*- Pada ResultLog terdiri 4 Digit , misal 0832, ini berarti 2 Digit di bagian depan nya adalah 08, bagian 2 Digit inilah yang harus kamu prediksi bahwa tidak akan keluar di periode berikut nya
*- Kamu harus percaya diri dalam memberikan jawaban list angka 2 Digit bagian depan nya, serta jangan ragu 
 
### FORMAR OUTPUT
List 2 Digit : [09,45,96,...] 
Alasan : {berikan alasan mengapa angka tersebut tidak akan keluar} 
 
### KONTEKS.
 PERIODE : 2235 - ResultLog : 4908
 Dicarikan 2 Digit Depan dari ResultLog 4908 (49) ke data history berikut data ResultLog di periode sebelum nya : 
 [{periode: 2216, resultLog: 6492}, {periode: 2203, resultLog: 4945}, {periode: 2186, resultLog: 0549}, {periode: 2154, resultLog: 7449}]
 Dicarikan 2 Digit Belakang dari ResultLog 4908 (08) ke data history berikut data ResultLog di periode sebelum nya : 
 [{periode: 2224, resultLog: 0821}, {periode: 2216, resultLog: 0108}, {periode: 2204, resultLog: 0833}, {periode: 2176, resultLog: 5088}]
 
 PERIODE : 2234 - ResultLog : 2581
 Dicarikan 2 Digit Depan dari ResultLog 2581 (25) ke data history berikut data ResultLog di periode sebelum nya : 
 [{periode: 2207, resultLog: 6252}, {periode: 2201, resultLog: 0625}, {periode: 2197, resultLog: 9625}, {periode: 2189, resultLog: 2250}]
 Dicarikan 2 Digit Belakang dari ResultLog 2581 (81) ke data history berikut data ResultLog di periode sebelum nya : 
 [{periode: 2216, resultLog: 8124}, {periode: 2212, resultLog: 6819}, {periode: 2191, resultLog: 3811}, {periode: 2167, resultLog: 1812}]
 `;

console.log('--- SFCore AI SSE TEST CLIENT ---');
console.log('Ketik pesan lalu ENTER (Ctrl+C untuk keluar)');

rl.question('> ', (userInput) => {
    const messages = [
        { role: 'system', content: systemPrompt },
        { role: 'user', content: userInput }
    ];

    sseChat(messages)
        .then((output) => {
            console.log('\n[Full Output Received]');
            rl.close();
        })
        .catch((err) => {
            console.error('\n[Fatal Error]', err.message);
            rl.close();
        });
});
