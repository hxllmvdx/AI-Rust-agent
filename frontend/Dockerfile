FROM python:3.12-slim

WORKDIR /app

COPY pyproject.toml ./
COPY .python-version ./

RUN pip install --no-cache-dir uv

RUN uv sync --no-dev

COPY app.py ./

EXPOSE 8501

CMD ["uv", "run", "streamlit", "run", "app.py", "--server.address=0.0.0.0", "--server.port=8501"]
