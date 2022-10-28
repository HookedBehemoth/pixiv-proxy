# Pixiv Proxy
An alternative frontend for pixiv.net

## Usage
`./reapixa --port 8080 --host https://example.org --cookie PHPSESSID=...`

If no cookie is provided, a guest cookie will be fetched.

## NGINX
It is recommended to add these nginx rules for caching, disallowing crawlers and forwarding image proxies.
```nginx
location = /robots.txt {
	add_header Content-Type text/plain;
	return 200 "User-agent: *\nDisallow: /";
}
location /imageproxy/ {
	proxy_pass https://i.pximg.net/;
	proxy_set_header Referer https://pixiv.net;
	proxy_hide_header Set-Cookie;
	proxy_buffering off;
}
location /simg/ {
	proxy_pass https://s.pximg.net/;
	proxy_set_header Referer https://pixiv.net;
	proxy_hide_header Set-Cookie;
	proxy_buffering off;
}
location /spix/ {
	proxy_pass https://img-sketch.pixiv.net/;
	proxy_set_header Referer https://sketch.pixiv.net;
	proxy_hide_header Set-Cookie;
	proxy_buffering off;
}
location /spxi/ {
	proxy_pass https://img-sketch.pximg.net/;
	proxy_set_header Referer https://sketch.pixiv.net;
	proxy_hide_header Set-Cookie;
	proxy_buffering off;
}
location /ugoira {
	proxy_pass http://127.0.0.1:8080/ugoira;
	proxy_cache STATIC;
	proxy_cache_valid 200 30d;
	add_header X-Cache-Status $upstream_cache_status;
}
location / {
	proxy_pass http://127.0.0.1:8080$request_uri;
	proxy_cache STATIC;
	proxy_cache_valid 200 3m;
	add_header X-Cache-Status $upstream_cache_status;
}
```
