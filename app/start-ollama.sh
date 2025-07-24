#!/bin/sh

MODEL="hf.co/openbmb/MiniCPM-o-2_6-gguf:Q4_K_M"

# Start Ollama server in the background
ollama serve &
OLLAMA_PID=$!

# Wait for Ollama server to be ready
echo "Waiting for Ollama server to be ready..."
until curl -s http://localhost:11434/api/tags > /dev/null; do
  sleep 1
done

# Only pull the model if it doesn't exist
if ! ollama list | grep -q "$MODEL"; then
  echo "Model $MODEL not found, pulling..."
  ollama pull "$MODEL"
else
  echo "Model $MODEL already present, skipping pull."
fi

echo "starting ollama server..."
# Wait for the Ollama server to exit (keep container running)
wait $OLLAMA_PID 