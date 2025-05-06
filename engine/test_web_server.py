from http.server import HTTPServer, SimpleHTTPRequestHandler

class COOPCOEPHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        self.send_header("Cross-Origin-Opener-Policy", "same-origin")
        super().end_headers()
    
    def send_head(self):
        if self.path == '/' or self.path == '/testpage.html' or self.path == '/tests/testpage.html':
            self.path = '/tests/testpage.html'
        return super().send_head()

if __name__ == "__main__":
    httpd = HTTPServer(("0.0.0.0", 8000), COOPCOEPHandler)
    print("Serving on http://localhost:8000 with COOP+COEP")
    httpd.serve_forever()