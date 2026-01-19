"""
Pre-Training Setup Script untuk Google Colab
=============================================

Jalankan script ini di cell pertama sebelum training.
Script ini akan:
1. Setup HuggingFace cache directory
2. Install dependencies dengan T4 GPU compatibility
3. Pre-download model ke cache

Usage di Colab:
    !python scripts/colab_setup.py
    
Atau copy-paste isi script ke cell pertama notebook.
"""

import os
import subprocess
import sys

# ===== KONSTANTA =====
HF_CACHE_DIR = "/content/hf_cache"
DEFAULT_MODEL = "google/gemma-3-270m-it"


def setup_cache_directory():
    """Setup HuggingFace cache directory untuk Colab."""
    print("📁 Setting up cache directories...")
    
    # Set environment variables
    os.environ['HF_HOME'] = HF_CACHE_DIR
    os.environ['TRANSFORMERS_CACHE'] = f"{HF_CACHE_DIR}/transformers"
    os.environ['HF_HUB_CACHE'] = f"{HF_CACHE_DIR}/hub"
    
    # Create directories
    os.makedirs(HF_CACHE_DIR, exist_ok=True)
    os.makedirs(f"{HF_CACHE_DIR}/transformers", exist_ok=True)
    os.makedirs(f"{HF_CACHE_DIR}/hub", exist_ok=True)
    
    print(f"   ✅ HF_HOME: {HF_CACHE_DIR}")
    print(f"   ✅ TRANSFORMERS_CACHE: {HF_CACHE_DIR}/transformers")
    print(f"   ✅ HF_HUB_CACHE: {HF_CACHE_DIR}/hub")


def install_dependencies():
    """Install dependencies dengan T4 GPU compatibility."""
    print("\n📦 Installing dependencies...")
    
    # Core dependencies
    core_deps = [
        "torch", "transformers", "accelerate", "bitsandbytes", 
        "peft", "trl", "datasets", "sentencepiece", "protobuf",
        "huggingface-hub", "wandb", "tensorboard", "psutil", 
        "pynvml", "pyyaml", "tqdm", "numpy"
    ]
    
    # Install core deps
    subprocess.run([
        sys.executable, "-m", "pip", "install", "-q", *core_deps
    ], check=True)
    print("   ✅ Core dependencies installed")
    
    # Install Unsloth (T4 GPU compatible)
    print("\n🚀 Installing Unsloth (T4 GPU compatible)...")
    subprocess.run([
        sys.executable, "-m", "pip", "install", "-q",
        "unsloth[colab-new] @ git+https://github.com/unslothai/unsloth.git"
    ], check=True)
    
    # Re-install deps tanpa conflicts (untuk T4 older architecture)
    subprocess.run([
        sys.executable, "-m", "pip", "install", "-q", "--no-deps",
        "trl", "peft", "accelerate", "bitsandbytes"
    ], check=True)
    print("   ✅ Unsloth installed (T4 compatible)")


def verify_gpu():
    """Verify GPU availability."""
    print("\n🎮 Verifying GPU...")
    
    try:
        import torch
        
        if torch.cuda.is_available():
            device_name = torch.cuda.get_device_name(0)
            cuda_version = torch.version.cuda
            print(f"   ✅ GPU: {device_name}")
            print(f"   ✅ CUDA: {cuda_version}")
            print(f"   ✅ PyTorch: {torch.__version__}")
            return True
        else:
            print("   ⚠️ GPU not available!")
            print("   💡 Tip: Runtime → Change runtime type → GPU")
            return False
    except ImportError:
        print("   ⚠️ PyTorch not installed yet")
        return False


def pre_download_model(model_name: str = DEFAULT_MODEL):
    """Pre-download model ke cache untuk faster startup."""
    print(f"\n📥 Pre-downloading model: {model_name}")
    
    try:
        from huggingface_hub import snapshot_download
        
        # Download model ke cache
        path = snapshot_download(
            repo_id=model_name,
            cache_dir=f"{HF_CACHE_DIR}/hub",
            ignore_patterns=["*.md", "*.txt"]  # Skip non-essential files
        )
        print(f"   ✅ Model cached to: {path}")
        return path
    except ImportError:
        print("   ⚠️ huggingface_hub not installed, skipping pre-download")
        return None
    except Exception as e:
        print(f"   ⚠️ Pre-download failed: {e}")
        print("   💡 Model akan di-download saat from_pretrained()")
        return None


def main():
    """Main setup function."""
    print("=" * 60)
    print("🚀 COLAB PRE-TRAINING SETUP")
    print("=" * 60)
    
    # 1. Setup cache
    setup_cache_directory()
    
    # 2. Verify GPU (before installing deps)
    gpu_ok = verify_gpu()
    
    # 3. Install dependencies
    install_dependencies()
    
    # 4. Pre-download model (optional)
    # Uncomment untuk pre-download:
    # pre_download_model("google/gemma-3-270m-it")
    
    print("\n" + "=" * 60)
    print("✅ SETUP COMPLETE!")
    print("=" * 60)
    print("\n📋 Next steps:")
    print("   1. Upload dataset: files.upload()")
    print("   2. Run training cells")
    print(f"\n📁 Cache location: {HF_CACHE_DIR}")
    
    if gpu_ok:
        print("🎮 GPU ready for training!")
    else:
        print("⚠️ Please enable GPU before training!")


if __name__ == "__main__":
    main()
