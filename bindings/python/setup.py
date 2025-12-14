from setuptools import setup, find_packages
import platform
import os
import shutil

# Determine the library file name based on platform
if platform.system() == "Darwin":
    lib_name = "libtoonify.dylib"
elif platform.system() == "Windows":
    lib_name = "toonify.dll"
else:
    lib_name = "libtoonify.so"

# Read the README file
readme_path = os.path.join(os.path.dirname(__file__), "README.md")
long_description = ""
if os.path.exists(readme_path):
    with open(readme_path, "r", encoding="utf-8") as f:
        long_description = f.read()

# Create toonifypy package directory if it doesn't exist
pkg_dir = os.path.join(os.path.dirname(__file__), "toonifypy")
os.makedirs(pkg_dir, exist_ok=True)

# Copy toonify.py to package as __init__.py
src_py = os.path.join(os.path.dirname(__file__), "toonify.py")
dst_py = os.path.join(pkg_dir, "__init__.py")
if os.path.exists(src_py):
    shutil.copy2(src_py, dst_py)

# Copy native library to package directory
src_lib = os.path.join(os.path.dirname(__file__), lib_name)
dst_lib = os.path.join(pkg_dir, lib_name)
if os.path.exists(src_lib):
    shutil.copy2(src_lib, dst_lib)

setup(
    name="toonifypy",
    version="1.1.8",
    description="High-performance JSON ↔ TOON converter. Reduce LLM token usage by 30-60% with Rust-powered Python bindings.",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="TOONify Contributors",
    author_email="",
    license="MIT",
    url="https://github.com/npiesco/TOONify",
    project_urls={
        "Bug Tracker": "https://github.com/npiesco/TOONify/issues",
        "Documentation": "https://github.com/npiesco/TOONify#readme",
        "Source Code": "https://github.com/npiesco/TOONify",
    },
    packages=["toonifypy"],
    package_data={"toonifypy": [lib_name, "*.dylib", "*.so", "*.dll"]},
    include_package_data=True,
    python_requires=">=3.8",
    keywords=["toon", "json", "converter", "llm", "ai", "token-optimization", "rust", "uniffi"],
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Rust",
        "Topic :: Software Development :: Libraries",
        "Topic :: Text Processing",
        "Operating System :: OS Independent",
    ],
)

