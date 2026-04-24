import os
from typing import Any

import requests
import streamlit as st

BACKEND_URL = os.getenv("BACKEND_URL", "http://localhost:8080").rstrip("/")


def backend_post(
    path: str, payload: dict[str, Any] | None = None, timeout: int = 60
) -> dict[str, Any]:
    response = requests.post(
        f"{BACKEND_URL}{path}", json=payload, timeout=timeout
    )
    response.raise_for_status()
    return response.json()


def backend_get(path: str, timeout: int = 30) -> dict[str, Any]:
    response = requests.get(f"{BACKEND_URL}{path}", timeout=timeout)
    response.raise_for_status()
    return response.json()


def ensure_session() -> None:
    if "session_id" not in st.session_state:
        data = backend_post("/sessions")
        st.session_state.session_id = data["session_id"]

    if "messages" not in st.session_state:
        st.session_state.messages = []

    if "last_used_tools" not in st.session_state:
        st.session_state.last_used_tools = []

    if "backend_url" not in st.session_state:
        st.session_state.backend_url = BACKEND_URL


def reset_session() -> None:
    session_id = st.session_state.session_id
    backend_post(f"/reset/{session_id}")
    st.session_state.messages = []
    st.session_state.last_used_tools = []
    st.toast("Session reset")


def sync_history_from_backend() -> None:
    session_id = st.session_state.session_id
    data = backend_get(f"/history/{session_id}")
    st.session_state.messages = data.get("messages", [])


def send_message(user_text: str) -> None:
    session_id = st.session_state.session_id

    st.session_state.messages.append({"role": "user", "content": user_text})

    try:
        data = backend_post(
            "/chat",
            {"session_id": session_id, "message": user_text},
            timeout=1200,
        )

        answer = data["answer"]
        used_tools = data.get("used_tools", [])

        st.session_state.messages.append(
            {"role": "assistant", "content": answer}
        )
        st.session_state.last_used_tools = used_tools

    except requests.HTTPError as exc:
        error_text = f"Backend returned HTTP {exc.response.status_code}"
        st.session_state.messages.append(
            {"role": "assistant", "content": error_text}
        )
        st.session_state.last_used_tools = []
    except requests.RequestException as exc:
        error_text = f"Failed to reach backend: {exc}"
        st.session_state.messages.append(
            {"role": "assistant", "content": error_text}
        )
        st.session_state.last_used_tools = []


st.set_page_config(page_title="Rust AI Agent", page_icon="🦀", layout="wide")

ensure_session()

st.title("Rust AI Agent")
st.caption("A+C agent: local structured knowledge + GitHub API")

with st.sidebar:
    st.subheader("Connection")
    st.write(f"**Backend:** `{st.session_state.backend_url}`")
    st.write(f"**Session ID:** `{st.session_state.session_id}`")

    col1, col2 = st.columns(2)

    with col1:
        if st.button("New session", use_container_width=True):
            data = backend_post("/sessions")
            st.session_state.session_id = data["session_id"]
            st.session_state.messages = []
            st.session_state.last_used_tools = []
            st.rerun()

    with col2:
        if st.button("Reset", use_container_width=True):
            reset_session()
            st.rerun()

    if st.button("Reload history", use_container_width=True):
        sync_history_from_backend()
        st.rerun()

    st.divider()
    st.subheader("Last tools used")
    if st.session_state.last_used_tools:
        for tool_name in st.session_state.last_used_tools:
            st.code(tool_name)
    else:
        st.write("No tools used yet.")

    st.divider()
    st.subheader("Example prompts")
    st.markdown(
        "- Compare Axum and Actix for a new Rust backend\n"
        "- Compare Axum and Actix and show active GitHub repos\n"
        "- Find active Rust observability repositories\n"
        "- Say hello in one sentence"
    )

for message in st.session_state.messages:
    role = message.get("role", "assistant")
    content = message.get("content", "")

    if role == "tool":
        with st.expander("Tool activity", expanded=False):
            st.write(content)
    else:
        with st.chat_message("user" if role == "user" else "assistant"):
            st.markdown(content)

prompt = st.chat_input("Ask about Rust backend tooling, repos, or trade-offs")

if prompt:
    with st.chat_message("user"):
        st.markdown(prompt)

    with st.chat_message("assistant"):
        with st.spinner("Thinking..."):
            send_message(prompt)
            st.markdown(st.session_state.messages[-1]["content"])

    st.rerun()
