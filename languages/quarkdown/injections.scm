((inline) @injection.content
  (#set! injection.language "quarkdown-inline"))

(fenced_code_block
  (info_string
    (language) @injection.language)
  (code_fence_content) @injection.content)

((minus_metadata) @injection.content
  (#set! injection.language "yaml")
  (#offset! @injection.content 1 0 -1 0)
  (#set! injection.include-children))

((plus_metadata) @injection.content
  (#set! injection.language "toml")
  (#offset! @injection.content 1 0 -1 0)
  (#set! injection.include-children))
