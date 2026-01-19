
def _format_messages_to_text(messages):
    """
    Converts a list of message dictionaries into a single string for tokenization.
    Assumes message format: [{"role": "system|user|assistant", "content": "..."}]
    Handles cases where messages might be empty or None.
    """
    if not messages:
        return ""

    formatted_text = []
    for msg in messages:
        role = msg.get("role", "user") # Default to user if role not specified
        content = msg.get("content", "")
        formatted_text.append(f"{role}: {content}")
    return "\n".join(formatted_text)

def analyze_dataset_and_configure(dataset, tokenizer, max_length=1024, vram_gb=16.0):
    print("🔍 Analyzing dataset...")
    print(f"   Initial dataset columns: {dataset.column_names}")

    # Ensure 'messages' column exists. If not, can't proceed with this strategy.
    if 'messages' not in dataset.column_names:
        raise ValueError("Dataset does not contain a 'messages' column. This script expects conversational data.")

    # Always ensure 'text' is generated from 'messages' for consistency
    # This proactively creates/overwrites 'text' if 'messages' exists
    print("   Proactively converting 'messages' to 'text' for length analysis...")
    dataset = dataset.map(
        lambda x: {"text": _format_messages_to_text(x.get("messages", []))},
        num_proc=os.cpu_count() // 2 or 1,
        desc="Formatting messages to text",
        load_from_cache_file=False # Force recomputation
    )
    print("   Conversion complete.")
    print(f"   Dataset columns after 'messages' to 'text' conversion: {dataset.column_names}")

    # Final check: 'text' column MUST exist at this point.
    if 'text' not in dataset.column_names:
        raise ValueError("FATAL: 'text' column was not created by the 'messages' conversion. Please check your dataset structure.")

    # Tokenize and calculate lengths
    print(f"   Columns before token length calculation: {dataset.column_names}")
    dataset = dataset.map(
        lambda x: {"length": len(tokenizer.encode(x["text"], add_special_tokens=False))},
        num_proc=os.cpu_count() // 2 or 1,
        desc="Calculating token lengths",
        load_from_cache_file=False # Force recomputation
    )
    print(f"   Columns after token length calculation: {dataset.column_names}")


    # Find the 95th percentile length
    lengths = sorted(dataset["length"])
    p95_length = lengths[int(len(lengths) * 0.95)]
    print(f"   95th percentile token length: {p95_length:,}")

    # Determine max_seq_length
    determined_max_seq_length = min(max_length, p95_length + 256)
    determined_max_seq_length = max(determined_max_seq_length, 512) # Minimum reasonable length

    # Dynamic configuration based on VRAM and sequence length
    per_device_train_batch_size = 1
    gradient_accumulation_steps = 16 # Good starting point for 0.6B on T4

    if vram_gb >= 16.0: # T4 GPU
        if determined_max_seq_length <= 1024:
            per_device_train_batch_size = 2
            gradient_accumulation_steps = 8 # Effective batch = 16
        elif determined_max_seq_length <= 2048:
            per_device_train_batch_size = 1
            gradient_accumulation_steps = 16 # Effective batch = 16
        elif determined_max_seq_length <= 4096:
            per_device_train_batch_size = 1
            gradient_accumulation_steps = 32 # Effective batch = 32
        else: # very long sequences, use more accumulation
            per_device_train_batch_size = 1
            gradient_accumulation_steps = 64 # Effective batch = 64 (if possible)
            determined_max_seq_length = min(determined_max_seq_length, 8192) # Cap for safety

    effective_batch_size = per_device_train_batch_size * gradient_accumulation_steps

    dynamic_config = {
        "max_seq_length": determined_max_seq_length,
        "per_device_train_batch_size": per_device_train_batch_size,
        "gradient_accumulation_steps": gradient_accumulation_steps,
        "effective_batch_size": effective_batch_size,
        "use_gradient_checkpointing": True, # Always good for memory with large models/sequences
    }

    print(f"   Determined Max Seq Length: {dynamic_config['max_seq_length']:,}")
    print(f"   Per Device Train Batch Size: {dynamic_config['per_device_train_batch_size']}")
    print(f"   Gradient Accumulation Steps: {dynamic_config['gradient_accumulation_steps']}")
    print(f"   Effective Batch Size: {dynamic_config['effective_batch_size']}")
    print(f"   Using Gradient Checkpointing: {dynamic_config['use_gradient_checkpointing']}")

    return dataset, dynamic_config
