get() {
    path=$1
    curl -X GET "$api_url/$path" | jq .
}

post() {
    path=$1
    body_dir=$2
    body=$3
    file="$scripts/$body_dir/$body.json"

    curl -d "@$file" -X POST -H "Content-Type: application/json" "$api_url/$path" | jq .
}

get_body() {
    printf '%s\n' "$@" | fzf
}
