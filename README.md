This is an implementation of the barnes-hut algorithm, which allows simulating n body systems in O(n log n) time.
It also includes a a simple particle renderer for the "stars".

# ⚠️ Warning ⚠️
The actual algorithm is technically broken. There exists a small ghost force, which acts on particles in dense regions of the quad tree.
This is not super obvious though, and it still produces pretty pictures, which is all I really wrote it for anyways :p
Also, this is a toy project, so a lot of magic constants, etc, etc.
