#!/usr/bin/env bb

(require '[log])

(let [cargo-toml (slurp "Cargo.toml")
      readme (slurp "README.md")
      cargo-version (second (re-find #"version\s*=\s*\"(\d+\.\d+\.\d+)\""
                                     cargo-toml))
      tag (str "v" cargo-version)
      readme-tags (map second (re-seq #"refs/tags/(v\d+\.\d+\.\d+)" readme))
      mismatched (remove #(= tag %) readme-tags)]
  (if (seq mismatched)
    (do (log/error (str "Cargo.toml version: " cargo-version))
        (doseq [t mismatched]
          (log/error (str "README.md has outdated tag: " t)))
        (System/exit 1))
    (log/ok (str "All tags match " tag))))
