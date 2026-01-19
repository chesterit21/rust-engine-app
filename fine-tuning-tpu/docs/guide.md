Berdasarkan riset mendalam terhadap praktik terbaik PyTorch XLA dan TensorFlow TPU, berikut adalah panduan yang telah disempurnakan untuk mengonversi kode fine-tuning dari GPU ke TPU di Google Colab. [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/run-calculation-pytorch)

## Setup Runtime TPU

Pastikan runtime diubah terlebih dahulu: **Runtime > Change Runtime Type > Hardware accelerator > TPU**. [tensorflow](https://www.tensorflow.org/guide/distributed_training)

## TensorFlow TPU Strategy

```python
import tensorflow as tf

# Initialize TPU dengan error handling
try:
    resolver = tf.distribute.cluster_resolver.TPUClusterResolver(tpu='')
    tf.config.experimental_connect_to_cluster(resolver)
    tf.tpu.experimental.initialize_tpu_system(resolver)
    strategy = tf.distribute.TPUStrategy(resolver)
    print(f"TPU devices: {tf.config.list_logical_devices('TPU')}")
    print(f"Number of replicas: {strategy.num_replicas_in_sync}")
except ValueError:
    raise RuntimeError("TPU tidak terdeteksi. Pastikan runtime type sudah diubah ke TPU")

# Model, optimizer, dan metrics HARUS dibuat dalam strategy.scope()
with strategy.scope():
    model = tf.keras.models.Sequential([
        # layer model Anda di sini
    ])
    
    # Compile di dalam scope
    model.compile(
        optimizer='adam',
        loss='sparse_categorical_crossentropy',
        metrics=['accuracy']
    )

# Training dengan batch size yang optimal untuk TPU
BATCH_SIZE_PER_REPLICA = 128  # Kelipatan 128 optimal untuk TPU
global_batch_size = BATCH_SIZE_PER_REPLICA * strategy.num_replicas_in_sync

# Dataset harus didistribusikan dengan strategy
train_dataset = ... # your dataset here
train_dataset = train_dataset.batch(global_batch_size)
dist_dataset = strategy.experimental_distribute_dataset(train_dataset)

# Training berjalan normal
model.fit(dist_dataset, epochs=10)
```

**Referensi:** [TensorFlow Distributed Training Guide](https://www.tensorflow.org/guide/distributed_training) [tensorflow](https://www.tensorflow.org/guide/distributed_training)

## PyTorch TPU dengan XLA

```python
# Install dependencies (jalankan di cell pertama)
!pip install cloud-tpu-client==0.10 torch==2.0.0 https://storage.googleapis.com/pytorch-xla-releases/wheels/tpuvm/torch_xla-2.0-cp310-cp310-linux_x86_64.whl

import torch
import torch_xla
import torch_xla.core.xla_model as xm
import torch_xla.distributed.parallel_loader as pl
import torch_xla.distributed.xla_multiprocessing as xmp
from torch_xla.amp import syncfree

def train_fn(index):
    """Training function yang akan di-spawn ke semua TPU cores"""
    
    # Get XLA device
    device = xm.xla_device()
    
    # Model dan optimizer
    model = YourModel().to(device)
    
    # Gunakan sync-free optimizer untuk performa lebih baik
    optimizer = syncfree.AdamW(model.parameters(), lr=5e-5)
    
    # Batch size harus kelipatan 8 atau 128
    BATCH_SIZE = 128
    train_loader = torch.utils.data.DataLoader(
        train_dataset,
        batch_size=BATCH_SIZE,
        shuffle=True,
        num_workers=4
    )
    
    # Wrap dataloader dengan ParallelLoader untuk preload ke TPU
    para_loader = pl.MpDeviceLoader(train_loader, device)
    
    # Mixed precision training dengan bfloat16 (TPU native)
    from torch_xla.amp import autocast
    
    model.train()
    for epoch in range(num_epochs):
        for batch_idx, (data, target) in enumerate(para_loader):
            optimizer.zero_grad()
            
            # Autocast untuk mixed precision
            with autocast(device):
                output = model(data)
                loss = loss_fn(output, target)
            
            loss.backward()
            
            # PENTING: Gunakan xm.optimizer_step untuk sync gradients
            xm.optimizer_step(optimizer)
            
            # Logging setiap N steps (hindari terlalu sering)
            if batch_idx % 10 == 0:
                xm.add_step_closure(
                    lambda: print(f'Epoch {epoch}, Step {batch_idx}, Loss: {loss.item()}')
                )
        
        # Checkpoint di akhir epoch
        if xm.is_master_ordinal():
            checkpoint = {
                'epoch': epoch,
                'model_state_dict': model.state_dict(),
                'optimizer_state_dict': optimizer.state_dict(),
            }
            # Save ke Google Drive
            xm.save(checkpoint, f'/content/drive/MyDrive/checkpoint_epoch_{epoch}.pt')

# Spawn training ke semua TPU cores (8 cores untuk TPU v2/v3)
if __name__ == '__main__':
    xmp.spawn(train_fn, nprocs=8, start_method='fork')
```

**Referensi:** [PyTorch XLA Documentation](https://pytorch.org/xla/release/r2.8/learn/xla-overview.html) [github](https://github.com/pytorch/xla)

## Optimasi Krusial untuk TPU

### Batch Size

- TPU memiliki 8 cores, gunakan batch size yang **divisible by 128** untuk efisiensi maksimal [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/performance-guide)
- Global batch size = `batch_size_per_replica × num_replicas` [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/performance-guide)
- Contoh optimal: 128, 256, 512, 1024 [towardsdatascience](https://towardsdatascience.com/a-comprehensive-guide-to-training-cnns-on-tpu-1beac4b0eb1c/)

### Mixed Precision Training

- TPU **hanya mendukung bfloat16**, bukan fp16 [lightning](https://lightning.ai/docs/pytorch/1.5.9/advanced/mixed_precision.html)
- BFloat16 memberikan performa 4-47% lebih cepat [cloud.google](https://cloud.google.com/blog/products/ai-machine-learning/bfloat16-the-secret-to-high-performance-on-cloud-tpus)
- Untuk PyTorch: gunakan `torch_xla.amp.autocast(device)` [docs.pytorch](https://docs.pytorch.org/xla/master/learn/migration-to-xla-on-tpus.html)
- Untuk TensorFlow: otomatis dengan `precision=16` di TPUStrategy [lightning](https://lightning.ai/docs/pytorch/1.5.9/advanced/mixed_precision.html)

### Data Loading

- Gunakan `MpDeviceLoader` (PyTorch) atau `experimental_distribute_dataset` (TF) untuk preload data ke TPU [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/run-calculation-pytorch)
- Hindari bottleneck I/O dengan `num_workers=4` atau lebih [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/run-calculation-pytorch)
- Prefetch data: `dataset.prefetch(tf.data.AUTOTUNE)` [towardsdatascience](https://towardsdatascience.com/a-comprehensive-guide-to-training-cnns-on-tpu-1beac4b0eb1c/)

### Graph Optimization

- **Hindari dynamic shapes** - gunakan padding untuk fixed shapes [docs.pytorch](https://docs.pytorch.org/xla/master/learn/migration-to-xla-on-tpus.html)
- Gunakan `torch_xla.sync()` atau `xm.mark_step()` untuk break graph di tempat yang tepat [docs.pytorch](https://docs.pytorch.org/xla/release/r2.8/learn/xla-overview.html)
- Wrap training loop dengan `@torch_xla.compile` jika memungkinkan [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/run-calculation-pytorch)

### Checkpoint & Logging

- **Reduce logging frequency** - device-host communication sangat mahal [docs.pytorch](https://docs.pytorch.org/xla/master/learn/migration-to-xla-on-tpus.html)
- Gunakan `xm.add_step_closure()` untuk async logging [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/run-calculation-pytorch)
- Save checkpoint ke Google Drive atau GCS bucket [stackoverflow](https://stackoverflow.com/questions/53017571/is-there-a-decent-workaround-to-saving-checkpoints-in-local-drive-when-using-tpu)
- Untuk TensorFlow: gunakan `tf.train.CheckpointManager` dengan localhost strategy [stackoverflow](https://stackoverflow.com/questions/53017571/is-there-a-decent-workaround-to-saving-checkpoints-in-local-drive-when-using-tpu)

```python
# PyTorch - Save checkpoint dengan XLA
if xm.is_master_ordinal():
    xm.save(model.state_dict(), '/content/drive/MyDrive/model.pth')

# TensorFlow - Save dengan TPUStrategy
checkpoint = tf.train.Checkpoint(model=model, optimizer=optimizer)
manager = tf.train.CheckpointManager(
    checkpoint,
    '/content/drive/MyDrive/checkpoints',
    max_to_keep=3
)
manager.save()
```

### Profiling & Debugging

- Disable progress bars (tqdm) karena trigger host-device sync [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/run-calculation-pytorch)
- Gunakan `xm.rendezvous('barrier')` untuk debug sync issues [docs.pytorch](https://docs.pytorch.org/xla/release/r2.8/learn/xla-overview.html)
- Profile dengan TensorBoard: `torch_xla.debug.profiler` [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/run-calculation-pytorch)

## Perbedaan Kritis GPU vs TPU

| Aspek | GPU | TPU |
|-------|-----|-----|
| Execution | Eager | Lazy (graph-based)  [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/run-calculation-pytorch) |
| Precision | FP16 | BFloat16  [reddit](https://www.reddit.com/r/MachineLearning/comments/vndtn8/d_mixed_precision_training_difference_between/) |
| Batch Size | Flexible | Optimal: kelipatan 128  [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/performance-guide) |
| Compilation | Minimal | First run lambat (caching)  [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/run-calculation-pytorch) |
| Gradient Sync | Implicit | Explicit via `xm.optimizer_step()`  [docs.pytorch](https://docs.pytorch.org/xla/master/learn/migration-to-xla-on-tpus.html) |

## Checklist Verifikasi

- [ ] Runtime type = TPU
- [ ] Batch size kelipatan 128
- [ ] Model & optimizer dalam `strategy.scope()` (TF) atau `.to(xla_device)` (PyTorch)
- [ ] Dataset wrapped dengan parallel loader
- [ ] Mixed precision dengan bfloat16
- [ ] Logging diminimalkan
- [ ] Checkpoint ke persistent storage (GDrive/GCS)
- [ ] Verifikasi TPU detection: `print(xm.xla_device())` atau `tf.config.list_logical_devices('TPU')`

**Referensi Lengkap:**

- [Cloud TPU Performance Guide](https://docs.cloud.google.com/tpu/docs/performance-guide) [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/performance-guide)
- [PyTorch XLA Best Practices](https://pytorch.org/xla/release/r2.8/learn/xla-overview.html) [docs.cloud.google](https://docs.cloud.google.com/tpu/docs/run-calculation-pytorch)
- [TensorFlow TPU Strategy](https://www.tensorflow.org/guide/distributed_training) [tensorflow](https://www.tensorflow.org/guide/distributed_training)
- [BFloat16 on Cloud TPUs](https://cloud.google.com/blog/products/ai-machine-learning/bfloat16-the-secret-to-high-performance-on-cloud-tpus) [cloud.google](https://cloud.google.com/blog/products/ai-machine-learning/bfloat16-the-secret-to-high-performance-on-cloud-tpus)
