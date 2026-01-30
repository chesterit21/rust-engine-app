#!/usr/bin/env node
// tcp_client.js
const net = require('net');
const readline = require('readline');

function tcpChat(messages, options = {}) {
    const {
        stream = true,
        maxTokens = 1200,
        host = '127.0.0.1',
        port = 8765
    } = options;

    return new Promise((resolve, reject) => {
        const client = net.createConnection({ host, port }, () => {
            const request = JSON.stringify({
                messages,
                stream,
                max_tokens: maxTokens
            }) + '\n';

            client.write(request);
        });

        let buffer = '';
        let output = '';

        client.on('data', (data) => {
            buffer += data.toString();

            let lines = buffer.split('\n');
            buffer = lines.pop();

            for (const line of lines) {
                if (!line.trim()) continue;

                try {
                    const response = JSON.parse(line);

                    if (response.token) {
                        process.stdout.write(response.token);
                        output += response.token;
                    } else if (response.done) {
                        console.log('\n\n[Metrics]', response.metrics);
                        client.end();
                        resolve(output);
                    } else if (response.error) {
                        console.error('\n[Error]', response.error);
                        client.end();
                        reject(new Error(response.error));
                    }
                } catch (err) {
                    console.error('[JSON parse error]', err.message);
                }
            }
        });

        client.on('error', reject);
        client.on('end', () => resolve(output));
    });
}

/* =========================
   CLI INPUT DARI TERMINAL
   ========================= */

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

console.log('Ketik pesan lalu ENTER (Ctrl+C untuk keluar)');
rl.question('> ', (userInput) => {
    const messages = [
        {
            role: 'system',
            content:
                'Kamu adalah AI Expert di bidang pencarian pola atau pattern prediksi angka, mempunyai kemampuan untuk memprediksi angka berdasarkan pola atau pattern yang diberikan.' +
                'Nama Kamu adalah SFCore AI, kamu di developmen oleh SFCore Team.' +
                ' ' +
                '### RULES & TASK.' +
                '*- Teliti dan perhitungkan dengan cermat [KONTEKS] yang di berikan sebelum memberikan jawaban.' +
                '*- Periode tertinggi adalah periode terbaru, dan data ResultLog nya adalah data terbaru.' +
                '*- Tugas kamu memberikan 10 Angka berupa 2 Digit di bagian depan yang kemungkinan nya tidak akan keluar.' +
                '*- 2 Digit itu di mulai dari 00 sampai dengan 99. Jadi tidak ada 2 Digit yang lebih besar dari 99 dan tidak ada 2 Digit yang lebih kecil dari 00.' +
                '*- Pada ResultLog terdiri 4 Digit , misal 0832, ini berarti 2 Digit di bagian depan nya adalah 08, bagian 2 Digit inilah yang harus kamu prediksi bahwa tidak akan keluar di periode berikut nya' +
                '*- Kamu harus percaya diri dalam memberikan jawaban list angka 2 Digit bagian depan nya, serta jangan ragu ' +
                ' ' +
                '### FORMAR OUTPUT' +
                'List 2 Digit : [09,45,96,...] ' +
                'Alasan : {berikan alasan mengapa angka tersebut tidak akan keluar} ' +
                ' ' +
                '### KONTEKS.' +
                ' PERIODE : 2235 - ResultLog : 4908' +
                ' Dicarikan 2 Digit Depan dari ResultLog 4908 (49) ke data history berikut data ResultLog di periode sebelum nya : ' +
                ' [{periode: 2216, resultLog: 6492}, {periode: 2203, resultLog: 4945}, {periode: 2186, resultLog: 0549}, {periode: 2154, resultLog: 7449}]' +
                ' Dicarikan 2 Digit Belakang dari ResultLog 4908 (08) ke data history berikut data ResultLog di periode sebelum nya : ' +
                ' [{periode: 2224, resultLog: 0821}, {periode: 2216, resultLog: 0108}, {periode: 2204, resultLog: 0833}, {periode: 2176, resultLog: 5088}]' +
                ' ' +
                ' PERIODE : 2234 - ResultLog : 2581' +
                ' Dicarikan 2 Digit Depan dari ResultLog 2581 (25) ke data history berikut data ResultLog di periode sebelum nya : ' +
                ' [{periode: 2207, resultLog: 6252}, {periode: 2201, resultLog: 0625}, {periode: 2197, resultLog: 9625}, {periode: 2189, resultLog: 2250}]' +
                ' Dicarikan 2 Digit Belakang dari ResultLog 2581 (81) ke data history berikut data ResultLog di periode sebelum nya : ' +
                ' [{periode: 2216, resultLog: 8124}, {periode: 2212, resultLog: 6819}, {periode: 2191, resultLog: 3811}, {periode: 2167, resultLog: 1812}]' +
                ' '
        },
        {
            role: 'user',
            content: userInput
        }
    ];

    tcpChat(messages)
        .then((output) => {
            console.log('\n[Full Output]', output);
            rl.close();
        })
        .catch((err) => {
            console.error(err);
            rl.close();
        });
});


// Cara pakai
// node tcp_client.js


// Lalu:
// > kamu bisa node js dan nest js?
