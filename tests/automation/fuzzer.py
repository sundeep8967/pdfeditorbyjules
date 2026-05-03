import ctypes
import os
import random
import tempfile
import multiprocessing
import sys

# Define FFI signatures
class DocumentHandle(ctypes.Structure):
    pass

# Try to find the shared library
lib_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../pdf_engine_core/target/debug/libpdf_engine_core.so"))
if not os.path.exists(lib_path):
    # Try MacOS extension
    lib_path = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../pdf_engine_core/target/debug/libpdf_engine_core.dylib"))

if not os.path.exists(lib_path):
    print(f"Error: Shared library not found at {lib_path}")
    sys.exit(1)

# Minimal valid PDF
MINIMAL_PDF = b'''%PDF-1.4
1 0 obj
<< /Type /Catalog /Pages 2 0 R >>
endobj
2 0 obj
<< /Type /Pages /Kids [3 0 R] /Count 1 >>
endobj
3 0 obj
<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>
endobj
4 0 obj
<< /Length 21 >>
stream
BT
/F1 24 Tf
100 100 Td
(Hello World) Tj
ET
endstream
endobj
xref
0 5
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
0000000115 00000 n
0000000204 00000 n
trailer
<< /Size 5 /Root 1 0 R >>
startxref
276
%%EOF'''

def mutate(data: bytes) -> bytes:
    """Applies a random mutation to the byte array."""
    data_list = bytearray(data)
    mutation_type = random.choice(['bitflip', 'byte_replace', 'truncate', 'insert'])

    if len(data_list) == 0:
        return bytes(data_list)

    if mutation_type == 'bitflip':
        idx = random.randint(0, len(data_list) - 1)
        data_list[idx] ^= (1 << random.randint(0, 7))
    elif mutation_type == 'byte_replace':
        idx = random.randint(0, len(data_list) - 1)
        data_list[idx] = random.randint(0, 255)
    elif mutation_type == 'truncate':
        if len(data_list) > 10:
            cut = random.randint(1, len(data_list) // 2)
            data_list = data_list[:-cut]
    elif mutation_type == 'insert':
        idx = random.randint(0, len(data_list))
        data_list.insert(idx, random.randint(0, 255))

    return bytes(data_list)

def test_pdf(file_path: str):
    """Loads the library and attempts to open the PDF. Runs in a subprocess to catch segfaults."""
    lib = ctypes.CDLL(lib_path)

    # pdf_engine_open_document
    lib.pdf_engine_open_document.argtypes = [ctypes.c_char_p]
    lib.pdf_engine_open_document.restype = ctypes.POINTER(DocumentHandle)

    # pdf_engine_free_document
    lib.pdf_engine_free_document.argtypes = [ctypes.POINTER(DocumentHandle)]
    lib.pdf_engine_free_document.restype = None

    c_path = file_path.encode('utf-8')
    handle = lib.pdf_engine_open_document(c_path)
    if handle:
        lib.pdf_engine_free_document(handle)

    sys.exit(0) # Exit cleanly if no segfault

def run_fuzzer(iterations: int = 1000):
    print(f"Starting fuzzer for {iterations} iterations...")
    crashes = 0

    for i in range(iterations):
        mutated_pdf = MINIMAL_PDF
        # Apply 1 to 5 mutations
        for _ in range(random.randint(1, 5)):
            mutated_pdf = mutate(mutated_pdf)

        with tempfile.NamedTemporaryFile(delete=False, suffix=".pdf") as tmp:
            tmp.write(mutated_pdf)
            tmp_path = tmp.name

        p = multiprocessing.Process(target=test_pdf, args=(tmp_path,))
        p.start()
        p.join()

        if p.exitcode != 0:
            print(f"Crash detected on iteration {i}! Exit code: {p.exitcode}")
            print(f"Saved crashing PDF to {tmp_path}")
            crashes += 1
        else:
            os.remove(tmp_path)

        if (i + 1) % 100 == 0:
            print(f"Completed {i + 1} iterations. Crashes so far: {crashes}")

    print(f"Fuzzing complete. Total crashes: {crashes}")
    if crashes > 0:
        sys.exit(1)

if __name__ == "__main__":
    # If run locally as a quick test, do fewer iterations
    # In CI, we would do 10000
    iters = 10000 if os.environ.get("CI") else 500
    run_fuzzer(iters)
