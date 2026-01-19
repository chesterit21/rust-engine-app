#!/usr/bin/env python3
"""
Package Script untuk Fine-Tuning Project
=========================================

Script ini mempackage semua file yang diperlukan untuk upload ke Google Colab
menjadi satu file zip yang mudah diupload.

Output: upload_package.zip (berisi src.zip + train_data.jsonl)

Usage:
    python3 scripts/package_for_upload.py
    
    # Atau dengan path dataset custom:
    python3 scripts/package_for_upload.py --dataset data/custom_data.jsonl
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


def create_src_zip(project_root: Path, output_path: Path):
    """Create zip dari folder src/."""
    src_dir = project_root / "src"
    
    if not src_dir.exists():
        print(f"❌ Error: src/ folder tidak ditemukan di {src_dir}")
        sys.exit(1)
    
    print(f"📦 Creating src.zip...")
    
    # Create zip
    with zipfile.ZipFile(output_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
        for root, dirs, files in os.walk(src_dir):
            # Skip __pycache__ directories
            dirs[:] = [d for d in dirs if d != '__pycache__']
            
            for file in files:
                if file.endswith('.pyc'):
                    continue
                    
                file_path = Path(root) / file
                arcname = file_path.relative_to(project_root)
                zipf.write(file_path, arcname)
                print(f"   + {arcname}")
    
    size_kb = output_path.stat().st_size / 1024
    print(f"   ✅ src.zip created ({size_kb:.1f} KB)")
    return output_path


def package_dataset(dataset_path: Path, output_dir: Path):
    """Copy dataset ke output directory."""
    if not dataset_path.exists():
        print(f"❌ Error: Dataset tidak ditemukan: {dataset_path}")
        sys.exit(1)
    
    output_path = output_dir / dataset_path.name
    shutil.copy2(dataset_path, output_path)
    
    size_kb = output_path.stat().st_size / 1024
    print(f"📄 Dataset copied: {dataset_path.name} ({size_kb:.1f} KB)")
    return output_path


def create_final_package(temp_dir: Path, output_path: Path):
    """Create final upload package."""
    print(f"\n📦 Creating final package: {output_path.name}")
    
    with zipfile.ZipFile(output_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
        for file in temp_dir.iterdir():
            zipf.write(file, file.name)
            print(f"   + {file.name}")
    
    size_mb = output_path.stat().st_size / (1024 * 1024)
    print(f"\n✅ Package created: {output_path}")
    print(f"   Size: {size_mb:.2f} MB")


def main():
    parser = argparse.ArgumentParser(description="Package files untuk upload ke Colab")
    parser.add_argument(
        "--dataset", 
        type=str, 
        default="data/train_data.jsonl",
        help="Path ke dataset JSONL (default: data/train_data.jsonl)"
    )
    parser.add_argument(
        "--output",
        type=str,
        default="upload_package.zip",
        help="Output filename (default: upload_package.zip)"
    )
    args = parser.parse_args()
    
    project_root = get_project_root()
    dataset_path = project_root / args.dataset
    output_path = project_root / args.output
    
    print("=" * 60)
    print("📦 FINE-TUNING PACKAGE CREATOR")
    print("=" * 60)
    print(f"\n📁 Project root: {project_root}")
    print(f"📄 Dataset: {dataset_path}")
    print(f"📦 Output: {output_path}\n")
    
    # Create temp directory
    temp_dir = project_root / ".temp_package"
    temp_dir.mkdir(exist_ok=True)
    
    try:
        # 1. Create src.zip
        src_zip = temp_dir / "src.zip"
        create_src_zip(project_root, src_zip)
        
        # 2. Copy dataset
        package_dataset(dataset_path, temp_dir)
        
        # 3. Copy notebook
        notebook_path = project_root / "notebooks" / "training.ipynb"
        if notebook_path.exists():
            shutil.copy2(notebook_path, temp_dir / "training.ipynb")
            print(f"📓 Notebook copied: training.ipynb")
        
        # 4. Create final package
        create_final_package(temp_dir, output_path)
        
        print("\n" + "=" * 60)
        print("🎉 PACKAGING COMPLETE!")
        print("=" * 60)
        print("\n📋 Next steps:")
        print("   1. Buka Google Colab atau VS Code")
        print("   2. Upload file: upload_package.zip")
        print("   3. Extract dan jalankan notebook")
        print(f"\n📁 File lokasi: {output_path}")
        
    finally:
        # Cleanup temp directory
        shutil.rmtree(temp_dir, ignore_errors=True)


if __name__ == "__main__":
    main()
