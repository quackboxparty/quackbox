use std::fs;
use std::path::Path;

use super::*;

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const IMAGE_CAP: u64 = 100 * KB;
const MEDIA_CAP: u64 = 1 * MB;

/// Extension → media kind mapping.
fn ext_kind(ext: &str) -> Option<MediaKind> {
    match ext {
        "avif" | "gif" | "jpeg" | "jpg" | "png" | "svg" | "webp" => Some(MediaKind::Image),
        "flac" | "m4a" | "mp3" | "ogg" | "opus" | "wav" => Some(MediaKind::Audio),
        "mov" | "mp4" | "webm" => Some(MediaKind::Video),
        _ => None,
    }
}

pub(super) fn check_media_files(ds: &Dataset, issues: &mut Vec<LoadIssue>) {
    let media_dir = Path::new(&ds.data_dir).join("media");

    for entry in ds.questions.values() {
        for (media_ref, kind) in entry.item.media_refs() {
            check_one_media(media_ref, kind, &entry.file, &media_dir, issues);
        }
    }
}

fn check_one_media(
    media_ref: &str,
    kind: MediaKind,
    context_file: &str,
    media_dir: &Path,
    issues: &mut Vec<LoadIssue>,
) {
    let Some(sub) = media_ref.strip_prefix("local:") else {
        return;
    };
    let full = media_dir.join(sub);

    let meta = match fs::metadata(&full) {
        Ok(m) => m,
        Err(_) => {
            issues.push(LoadIssue {
                file: context_file.to_owned(),
                message: format!("media file missing: {media_ref}"),
                path: None,
            });
            return;
        }
    };

    if let Some(ext) = Path::new(sub).extension().and_then(|e| e.to_str()) {
        let ext_lower = ext.to_lowercase();
        if let Some(actual) = ext_kind(&ext_lower) {
            let ok = actual == kind || (actual == MediaKind::Video && kind == MediaKind::Audio);
            if !ok {
                issues.push(LoadIssue {
                    file: context_file.to_owned(),
                    message: format!(
                        "media kind mismatch: declared {kind:?} but extension .{ext_lower} is {actual:?} ({media_ref})"
                    ),
                    path: None,
                });
            }
        } else {
            issues.push(LoadIssue {
                file: context_file.to_owned(),
                message: format!("unknown media extension: .{ext_lower} ({media_ref})"),
                path: None,
            });
        }
    }

    let cap = if kind == MediaKind::Image {
        IMAGE_CAP
    } else {
        MEDIA_CAP
    };
    if meta.len() > cap {
        issues.push(LoadIssue {
            file: context_file.to_owned(),
            message: format!(
                "media file exceeds size cap ({}B > {cap}B): {media_ref}",
                meta.len()
            ),
            path: None,
        });
    }
}
