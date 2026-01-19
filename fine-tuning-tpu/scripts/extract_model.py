#!/usr/bin/env python3
"""
Extract dan Setup Model Hasil Training
=======================================

Script ini mengextract model yang di-download dari Colab
dan menyiapkannya untuk testing dan konversi GGUF.

Usage:
    python3 scripts/extract_model.py final_model.zip
    
    # Dengan custom output directory:
    python3 scripts/extract_model.py final_model.zip --output outputs/my_model
"""

import os
import sys
import shutil
import zipfile
import argparse
from pathlib import Path
from datetime import datetime


def get_project_root():
    """Get project root directory."""
    return Path(__file__).parent.parent


def extract_model(zip_path: Path, output_dir: Path):
    """Extract model zip ke output directory."""
    
    if not zip_path.exists():
        print(f"❌ Error: File tidak ditemukan: {zip_path}")
        sys.exit(1)
    
    # Create output directory
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"📦 Extracting: {zip_path.name}")
    print(f"📁 Target: {output_dir}")
    print()
    
    # Extract
    with zipfile.ZipFile(zip_path, 'r') as zipf:
        file_list = zipf.namelist()
        print(f"📄 Files in archive ({len(file_list)}):")
        
        for file in file_list[:10]:  # Show first 10
            print(f"   - {file}")
        
        if len(file_list) > 10:
            print(f"   ... dan {len(file_list) - 10} file lainnya")
        
        print()
        zipf.extractall(output_dir)
    
    # Verify extraction
    extracted_files = list(output_dir.rglob("*"))
    model_files = [f for f in extracted_files if f.is_file()]
    
    print(f"✅ Extracted {len(model_files)} files")
    
    # Check for essential files
    essential_files = ["config.json", "tokenizer_config.json"]
    for efile in essential_files:
        found = any(efile in str(f) for f in model_files)
        status = "✅" if found else "⚠️"
        print(f"   {status} {efile}")
    
    return output_dir


def show_model_info(model_dir: Path):
    """Show information about the extracted model."""
    print("\n" + "=" * 60)
    print("📊 MODEL INFORMATION")
    print("=" * 60)
    
    # Check for config.json
    config_path = model_dir / "config.json"
    if config_path.exists():
        import json
        with open(config_path) as f:
            config = json.load(f)
        
        print(f"\n📋 Model Config:")
        print(f"   Model type: {config.get('model_type', 'unknown')}")
        print(f"   Vocab size: {config.get('vocab_size', 'unknown')}")
        print(f"   Hidden size: {config.get('hidden_size', 'unknown')}")
        print(f"   Num layers: {config.get('num_hidden_layers', 'unknown')}")
    
    # Calculate total size
    total_size = sum(f.stat().st_size for f in model_dir.rglob("*") if f.is_file())
    size_mb = total_size / (1024 * 1024)
    print(f"\n💾 Total size: {size_mb:.2f} MB")
    
    # List adapter files (LoRA)
    adapter_files = list(model_dir.glob("adapter_*"))
    if adapter_files:
        print(f"\n🔧 LoRA Adapters found:")
        for af in adapter_files:
            print(f"   - {af.name}")


def main():
    parser = argparse.ArgumentParser(description="Extract model dari Colab download")
    parser.add_argument(
        "zip_file",
        type=str,
        help="Path ke file zip model (e.g., final_model.zip)"
    )
    parser.add_argument(
        "--output",
        type=str,
        default=None,
        help="Output directory (default: outputs/final_model)"
    )
    args = parser.parse_args()
    
    project_root = get_project_root()
    
    # Handle zip path
    zip_path = Path(args.zip_file)
    if not zip_path.is_absolute():
        # Check in project root and common download locations
        possible_paths = [
            project_root / args.zip_file,
            Path.home() / "Downloads" / args.zip_file,
            Path(args.zip_file)
        ]
        
        for pp in possible_paths:
            if pp.exists():
                zip_path = pp
                break
    
    # Set output directory
    if args.output:
        output_dir = Path(args.output)
        if not output_dir.is_absolute():
            output_dir = project_root / args.output
    else:
        # Default: outputs/final_model_YYYYMMDD_HHMMSS
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        output_dir = project_root / "outputs" / f"final_model_{timestamp}"
    
    print("=" * 60)
    print("📦 MODEL EXTRACTOR")
    print("=" * 60)
    
    # Extract
    extract_model(zip_path, output_dir)
    
    # Show info
    show_model_info(output_dir)
    
    print("\n" + "=" * 60)
    print("🎉 EXTRACTION COMPLETE!")
    print("=" * 60)
    print(f"\n📁 Model location: {output_dir}")
    print("\n📋 Next steps:")
    print(f"   1. Test model: python3 scripts/test_model.py {output_dir}")
    print(f"   2. Convert to GGUF: python3 scripts/convert_to_gguf.py {output_dir}")


if __name__ == "__main__":
    main()
