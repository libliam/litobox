# OCR模型目录

此目录用于存放 PaddleOCR PP-OCRv6 Tiny 模型文件。

## 模型文件

首次使用OCR功能时，模型会自动从CDN下载并缓存到浏览器IndexedDB中。

如需手动放置模型文件，请将以下文件放入此目录：

- `PP-OCRv6_tiny_det.onnx` - 文字检测模型
- `PP-OCRv6_tiny_rec.onnx` - 文字识别模型
- `PP-OCRv6_tiny_rec_keys.txt` - 识别字典

## 自动下载

模型默认从以下地址下载：

- 检测模型: `https://paddle-model-ecology.bj.bcebos.com/paddlex/official_pretrained/PP-OCRv6_tiny_det.onnx`
- 识别模型: `https://paddle-model-ecology.bj.bcebos.com/paddlex/official_pretrained/PP-OCRv6_tiny_rec.onnx`
