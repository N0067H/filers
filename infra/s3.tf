variable "bucket_name" {
  type = string
}

resource "aws_s3_bucket" "file_storage" {
    bucket = var.bucket_name
}

resource "aws_s3_bucket_lifecycle_configuration" "file_storage" {
  bucket = aws_s3_bucket.file_storage.id

  rule {
    id     = "abort-incomplete-multipart-uploads"
    status = "Enabled"

    filter {
      prefix = "files/"
    }

    abort_incomplete_multipart_upload {
      days_after_initiation = 7
    }
  }
}

output "bucket_name" {
    value = aws_s3_bucket.file_storage.bucket
}
