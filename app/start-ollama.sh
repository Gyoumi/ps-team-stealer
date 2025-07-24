#!/bin/sh

MODEL="hf.co/openbmb/MiniCPM-o-2_6-gguf:Q4_K_M"

if ! ollama list | grep -q "$MODEL"; then
  echo "Model $MODEL not found, pulling..."
  ollama pull "$MODEL"
else
  echo "Model $MODEL already present, skipping pull."
fi

echo "Starting Ollama server..."
exec ollama serve 