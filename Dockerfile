FROM python:3.11-slim

ENV PYTHONUNBUFFERED=1

WORKDIR /app

COPY app.py .

EXPOSE 7777

CMD ["python", "app.py"]
