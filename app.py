from http.server import BaseHTTPRequestHandler, HTTPServer
import threading
import time

class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-type", "text/plain")
        self.end_headers()
        self.wfile.write(b"Hello from GitHub! Attempt1")


def log_message():
    while True:
        print("Server is still running...")
        time.sleep(10)


if __name__ == "__main__":
    # Start logger thread
    threading.Thread(target=log_message, daemon=True).start()

    server = HTTPServer(("0.0.0.0", 8000), Handler)
    print("Server running on port 8000")

    server.serve_forever()
