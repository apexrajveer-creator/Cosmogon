#!/usr/bin/env python3
"""Verify and endian-convert the PDS4 MOLA 4-pixels/degree global products.
No resampling, new elevation values, or inferred observation counts are introduced.
Usage: python3 tools/process_mola.py --source-dir tools/data/mola [--fetch]
"""
import argparse
import array
import hashlib
import json
from pathlib import Path
import sys
import urllib.request
import xml.etree.ElementTree as ET

BASE = 'https://pds-geosciences.wustl.edu/mgs/urn-nasa-pds-mgs_mola_topography_derived/'
PRODUCTS = {
    'megt90n000cb': ('25f16fb7aaf857898dcf98bc4f841341a24f8b9f7e98453ca083bc45d897ca2c', 'mars_mola_elevation.bin'),
    'megc90n000cb': ('7e920ec43497b4beddf974650471600c7afd4916e2a8dd3500d6b14714365661', 'mars_mola_counts.bin'),
}
NS = {'p': 'http://pds.nasa.gov/pds4/pds/v1'}

def process(source, destination):
    destination.mkdir(parents=True, exist_ok=True)
    inventory = []
    for product, (expected, output) in PRODUCTS.items():
        raw = (source / (product + '.img')).read_bytes()
        if len(raw) != 1440 * 720 * 2 or hashlib.sha256(raw).hexdigest() != expected:
            raise ValueError(f'{product}: incorrect size or pinned source checksum')
        label = (source / (product + '.xml')).read_bytes()
        root = ET.fromstring(label)
        image = root.find('.//p:Array_2D_Image', NS)
        if image is None or image.findtext('p:Element_Array/p:data_type', namespaces=NS) != 'SignedMSB2':
            raise ValueError('Unexpected MOLA array type')
        if [int(a.findtext('p:elements', namespaces=NS)) for a in image.findall('p:Axis_Array', NS)] != [720, 1440]:
            raise ValueError('Unexpected MOLA axes')
        version = root.findtext('p:Identification_Area/p:version_id', namespaces=NS)
        if version != '1.1':
            raise ValueError('Review changed PDS label before updating this dataset')
        grid = array.array('h'); grid.frombytes(raw)
        if sys.byteorder == 'little': grid.byteswap()
        if product.startswith('megc') and min(grid) < 0:
            raise ValueError('Negative observation count')
        bounds = [min(grid), max(grid)]
        if sys.byteorder != 'little': grid.byteswap()
        converted = grid.tobytes()
        (destination / output).write_bytes(converted)
        (destination / (product + '.xml')).write_bytes(label)
        inventory.append({'product': product, 'label_version': version, 'source_url': BASE + 'meg004/' + product + '.img',
                          'source_sha256': expected, 'label_sha256': hashlib.sha256(label).hexdigest(),
                          'output': output, 'output_sha256': hashlib.sha256(converted).hexdigest(), 'range': bounds})
    metadata = {'id': 'mgs-mola-megdr-4ppd', 'version': '2003-grid-pds4-label-1.1',
                'doi': '10.17189/1z1b-kv84', 'provider': 'NASA PDS / MGS MOLA Team / Washington University Geosciences Node',
                'attribution': 'Smith, D. E., MGS MOLA MEGDR, NASA Planetary Data System',
                'rights': 'Public domain PDS archive data; see provider FAQ and source registry',
                'rights_evidence': 'https://pds-geosciences.wustl.edu/documents/ViewingPDSImagesAndExportingToOtherFormats.pdf',
                'shape': [720, 1440], 'cell_degrees': 0.25, 'grid': 'Pixel centres: 89.875 N, 0.125 E; north-to-south, east longitude',
                'coordinate_system': 'IAU2000 planetocentric latitude, positive-east longitude',
                'vertical_datum': 'GMM3 (mgm1025), degree/order 50 areoid; topography = radius minus areoid radius',
                'unit': 'm', 'counts_unit': 'observations per bin', 'processing': 'Exact big-to-little endian conversion only; bilinear sampling at runtime',
                'uncertainty_m': None, 'products': inventory}
    (destination / 'metadata.json').write_text(json.dumps(metadata, indent=2) + '\n')
    print(json.dumps(metadata, indent=2))

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-dir', type=Path, default=Path('tools/data/mola'))
    parser.add_argument('--output-dir', type=Path, default=Path('crates/cosmogon_sim/data/mola'))
    parser.add_argument('--fetch', action='store_true')
    args = parser.parse_args()
    if args.fetch:
        args.source_dir.mkdir(parents=True, exist_ok=True)
        for product in PRODUCTS:
            for suffix in ['img', 'xml']:
                url = BASE + 'meg004/' + product + '.' + suffix
                with urllib.request.urlopen(url, timeout=30) as response:
                    data = response.read(3 * 1024 * 1024 + 1)
                if len(data) > 3 * 1024 * 1024: raise ValueError('Source exceeds bounded download size')
                (args.source_dir / (product + '.' + suffix)).write_bytes(data)
    process(args.source_dir, args.output_dir)
