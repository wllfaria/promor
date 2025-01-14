#!/usr/bin/env bash

pages=("create" "list")

handle_pages() {
    case $(printf '%s\n' "${pages[@]}" | fzf) in
        "list") get "pages";;
        "create")
            body=$(get_body "valid")
            post "pages" "data/pages" $body
        ;;
    esac
}

