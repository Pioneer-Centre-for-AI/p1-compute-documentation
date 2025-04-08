# P1's cluster documentation

This repository contains the documentation for the P1's cluster. It is built using [MkDocs](https://www.mkdocs.org/)
with the [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/) theme. The documentation is hosted on
[GitHub Pages](https://pages.github.com/).

## Getting Started

### 1. Clone the Repository

```bash
git clone git@github.com:Pioneer-Centre-for-AI/p1-compute-documentation.git
cd p1-compute-documentation
```

### 2. Set Up Your Development Environment

Choose one of the following methods to set up your development environment:

#### Option A: Using Nix and Direnv (Recommended but arguably a bit involved)

If you use Nix and Direnv, you can simply allow the Direnv shell augmentation:

```bash
direnv allow
```

#### Option B: Using uv

If you have `uv` installed, you can set up the virtual environment:

```bash
uv sync
```

#### Option C: Using pip (Alternative)

If you don't have Nix or uv, you can use pip with the generated requirements.txt:

```bash
python -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
pip install -r requirements.txt
pip install -e .
```

### 3. Serve the Documentation Locally

Once your environment is set up, you can preview the documentation locally:

```bash
mkdocs serve
```

The documentation will be available at `http://127.0.0.1:8000`.

## Online Documentation

The documentation is also available online at:
<https://pioneer-centre-for-ai.github.io/p1-compute-documentation/>
