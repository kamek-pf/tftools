From the repo's root directory:
```
# Generate CSV
python python/xml_to_csv.py
python python/generate_tfrecord.py --csv_input=dataset/train_labels.csv --output_path=dataset/train.tfrecord --image_dir=dataset
```

---

Inspired by [this repository](https://github.com/datitran/raccoon_dataset), credits to  datitran, and [this video series](https://www.youtube.com/watch?v=COlbP62-B-U) from sentdex.
