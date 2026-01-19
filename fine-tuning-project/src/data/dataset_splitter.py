
import os
from datasets import Dataset

def split_dataset(dataset, train_ratio, val_ratio, test_ratio, seed=42):
    print("📊 Dataset Splitting:")
    total_samples = len(dataset)
    print("   Total samples: {}".format(total_samples))
    print("   Train ratio: {}% ".format(train_ratio*100))
    print("   Validation ratio: {}% ".format(val_ratio*100))
    print("   Test ratio: {}% ".format(test_ratio*100))

    assert abs(train_ratio + val_ratio + test_ratio - 1.0) < 1e-6, "Ratios must sum to 1.0"

    # First split into train_val and test
    train_val_ratio = train_ratio + val_ratio
    train_val_dataset, test_dataset = dataset.train_test_split(
        test_size=test_ratio, seed=seed
    ).values()

    # Then split train_val into train and validation
    train_dataset, val_dataset = train_val_dataset.train_test_split(
        test_size=val_ratio / train_val_ratio, seed=seed
    ).values()

    print("\n✅ Dataset Split Complete:")
    print("   Train: {} samples ({:.1f}%)".format(len(train_dataset), len(train_dataset)/total_samples*100))
    print("   Validation: {} samples ({:.1f}%)".format(len(val_dataset), len(val_dataset)/total_samples*100))
    print("   Test: {} samples ({:.1f}%)".format(len(test_dataset), len(test_dataset)/total_samples*100))

    return {
        "train": train_dataset,
        "validation": val_dataset,
        "test": test_dataset,
    }


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
        formatted_text.append("{}: {}".format(role, content))
    return "\n".join(formatted_text)


def analyze_split_distribution(dataset_dict, tokenizer):
    print("\n📊 Analyzing token length distribution across splits:")
    for split_name, dataset in dataset_dict.items():
        print("   Initial columns for {}: {}".format(split_name, dataset.column_names))
        # Always ensure 'text' is generated from 'messages' for consistency
        if 'messages' in dataset.column_names:
            temp_dataset = dataset.map(
                lambda x: {"text": _format_messages_to_text(x.get("messages", []))},
                num_proc=os.cpu_count() // 2 or 1,
                desc="Formatting messages for {}".format(split_name),
                load_from_cache_file=False # Force recomputation
            )
            print("   Columns after 'messages' to 'text' conversion for {}: {}".format(split_name, temp_dataset.column_names))
            # Final check to ensure 'text' column exists in temp_dataset before processing
            if 'text' not in temp_dataset.column_names:
                print("   WARNING: 'text' column not found in {} after messages conversion. Skipping length analysis for this split.".format(split_name))
                continue
            lengths = [len(tokenizer.encode(x["text"], add_special_tokens=False)) for x in temp_dataset]
        elif 'text' in dataset.column_names:
            lengths = [len(tokenizer.encode(x["text"], add_special_tokens=False)) for x in dataset]
        else:
            print("   Skipping {} due to missing 'text' or 'messages' column.".format(split_name))
            continue

        if not lengths:
            print("   {} (0 samples)".format(split_name))
            continue
        avg_len = sum(lengths) / len(lengths)
        max_len = max(lengths)
        min_len = min(lengths)
        print("   {}: Avg Length={:.0f}, Max Length={}, Min Length={} ({} samples)".format(split_name, avg_len, max_len, min_len, len(dataset)))
