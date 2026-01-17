#### Percobaan pertama (Quick Wins)

> ~llama_context:        CPU compute buffer size is 302.7500 MiB, matches expectation of 302.7500 MiB
> [metrics] ftl: 407 ms, tokens: 517, time: 63086 ms, speed: 8.25 tok/s
> [memory] rss: 15.1 -> 1292.8 MB

#### Percobaan ke dua

> ~llama_context:        CPU compute buffer size is 302.7500 MiB, matches expectation of 302.7500 MiB
> [metrics] ftl: 568 ms, tokens: 517, time: 81430 ms, speed: 6.39 tok/s
> [memory] rss: 15.1 -> 1292.3 MB

#### Percobaan ke 3

> ~llama_context:        CPU compute buffer size is 302.7500 MiB, matches expectation of 302.7500 MiB
> [metrics] ftl: 1789 ms, tokens: 517, time: 106697 ms, speed: 4.93 tok/s
> [memory] rss: 14.9 -> 1292.4 MB

#### Percobaan ke 4 (Native + BLAS + Threads=3)
>
> ~llama_context:        CPU compute buffer size is 302.7500 MiB
> [metrics] ftl: 399 ms, tokens: 512, time: 61352 ms, speed: 8.40 tok/s
> [memory] rss: 15.1 -> 1293.6 MB
