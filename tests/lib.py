import re
from pathlib import Path
from subprocess import run, CompletedProcess
from typing import Self

ArgList = tuple[str]


class Result:
    def __init__(self, dest: Path, res: CompletedProcess):
        self.dest = dest
        self._res = res

    def _strip_color_codes(self, text: str) -> str:
        return re.sub(r"\x1b\[[0-9;]*m", "", text)

    def returncode(self, code: int):
        res = self._res
        assert res.returncode == code, \
            f"Expected {code}, got {res.returncode} (stderr: {res.stderr!r})"
        return self

    def stdout_contains(self, text: str):
        stdout = self._strip_color_codes(self._res.stdout)
        assert text in stdout, f"Expected {text!r} to be in {stdout!r}"
        return self

    def not_stdout_contains(self, text: str):
        stdout = self._strip_color_codes(self._res.stdout)
        assert text not in stdout, f"Expected {text!r} to not be in {stdout!r}"
        return self

    def stderr_contains(self, text: str):
        stderr = self._strip_color_codes(self._res.stderr)
        assert text in stderr, f"Expected {text!r} to be in {stderr!r}"
        return self


class Stencil:
    def __init__(self, cmd: str) -> None:
        self._cmd = cmd
        self._stencil_args: ArgList = []
        self._command_args: ArgList = []
        self._dest: Path|None = None
        self._src: str|None = None

    @classmethod
    def init(cls) -> Self:
        return cls("init")

    @classmethod
    def plan(cls) -> Self:
        return cls("plan")

    @classmethod
    def apply(cls) -> Self:
        return cls("apply")

    def dest(self, dest: Path) -> Self:
        self._dest = dest
        return self

    def src(self, src: str) -> Self:
        self._src = src
        return self

    def override(self, name: str, value: str) -> Self:
        self._stencil_args.extend(["--override", f"{name}={value}"])
        return self

    def arg(self, name: str, value: str) -> Self:
        self._command_args.extend([name, value])
        return self

    def run(self) -> Result:
        cmd = ["/stencil/stencil"]
        cmd.extend(self._stencil_args)
        cmd.append(self._cmd)
        cmd.extend(self._command_args)
        if self._dest:
            cmd.append(f"{self._dest}")
        if self._src:
            cmd.append(self._src)
        with open("/tests/ou.txt", "a") as f:
            f.write(f"CMD: {cmd}\n")
        print("CMD", cmd)
        return Result(self._dest, run(cmd, capture_output=True, text=True))


def slurp(directory: Path) -> dict[str, str]:
    rv = {}
    for path in directory.rglob("*"):
        if path.is_file():
            rv[str(path.relative_to(directory))] = path.read_text()
    return rv
