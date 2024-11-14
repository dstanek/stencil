FROM python:3.12-slim
WORKDIR /tests
COPY requirements.txt /tests/
RUN \
    pip install --upgrade pip && \
    pip install -r requirements.txt
CMD ["pytest", "tests.py", "-vv"]