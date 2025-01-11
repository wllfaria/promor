#!/usr/bin/env bash

stores=("list" "create")

handle_stores() {
    case $(printf '%s\n' "${stores[@]}" | fzf) in
        "list") get "stores";;
        "create")
            body=$(get_body "invalid_url" "valid" "invalid_name")
            post "stores" "data/stores" $body
        ;;
    esac
}
