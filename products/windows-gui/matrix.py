"""Select only pinned candidates; give each application a disposable runner."""
import json
import os
from pathlib import Path

from fetch import validate

rows = json.loads(Path(__file__).with_name('candidates.json').read_text())
for row in rows:
    validate(row)
products = [row['product'] for row in rows]
assert products and len(products) == len(set(products))
requested = os.environ.get('PRODUCT', 'all') or 'all'
assert requested == 'all' or requested in products, 'No native-passed candidate for this product'
selected = products if requested == 'all' else [requested]
with open(os.environ['GITHUB_OUTPUT'], 'a') as output:
    output.write('products=' + json.dumps(selected) + '\n')
print('Isolated native GUI candidates:', ', '.join(selected))
