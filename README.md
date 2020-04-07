# tftools
TensorFlow helpers, focus on ML, not on glue code.

## Use cases and examples
Heres what's currently available:
- Dataset preparation - object detection

### Dataset preparation - object detection
Labeled data is often presented in the form of XML files (PASCAL VOC). \
This is what annotation software like [`labelImg`](https://github.com/tzutalin/labelImg) outputs. \
In a typical project, as soon as you're done assembling and labeling your images, you'll need to :
- Split your dataset in two parts (for training and testing)
- Generate a label map file for TensorFlow (where each label is associated with a number)
- Generate `tfrecord` files for your each dataset to train your model

Just dump all your files and their labels in a single directory and `tftools` will generate all those files:

```
tftools prepare pascal-voc \
    --input-dirs /path/to /your /datasets \
    --output-dir /another/path
```

You'll end up with the following files in your output directory:
```
train.tfrecord    # contains 80% of your dataset by default
test.tfrecord     # contains 20% of your dataset by default
label-map.pbtxt   # label map generated from your XML files
```

The only requirement is that for each input file, there's a corresponding XML file with the same name. \
For instance, `input1.jpg` should have a `input1.xml` in the same directory. \
Check `tftools prepare pascal-voc --help` for more options.


## Technical details
The following section contains technical details for maintainers/contributors. \
If you're a user, you don't need to read this.

### Object detection API
The object detection API currently relies on the following attributes:
```bash
'image/height': int64
'image/width': int64
'image/filename': bytes
'image/source_id': bytes
'image/encoded': bytes
'image/format': bytes
'image/object/bbox/xmin': float_list
'image/object/bbox/xmax': float_list
'image/object/bbox/ymin': float_list
'image/object/bbox/ymax': float_list
'image/object/class/text': bytes_list
'image/object/class/label': int64_list
```

More information can be found [here](https://github.com/tensorflow/models/blob/master/research/object_detection/g3doc/using_your_own_dataset.md).

## Resources
- A [Medium post](https://medium.com/mostly-ai/tensorflow-records-what-they-are-and-how-to-use-them-c46bc4bbb564) on the tfrecord format.
