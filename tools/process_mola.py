#!/usr/bin/env python3
"""Verify and endian-convert the PDS4 MOLA 4- or 16-pixels/degree global products.
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

# The 16ppd products are separately pinned; updating a provider file requires review.
PRODUCTS16 = {
    'megt90n000eb': ('d18d9b9ab8c5516d02e157dd2cde0f1d0d160c21940e953ba22391269a545e7b', 'c393221db2de31916b30113ce28841dd967456dadb9bf2735eb30496eb37afa2', 'mars_mola_elevation.bin'),
    'megc90n000eb': ('9a300c6cd9f5a282ae31299be228ea2e75993049e88c9f6ab69a84f4db5f01d7', '845eda1aa007144821f3915e3c45dada4fbccb4f862de08f761c80406a156c28', 'mars_mola_counts.bin'),
}
INVENTORY = 'urn-nasa-pds-mgs_mola_topography_derived.md5'
INVENTORY_SHA = '4468a26e18e382896cb3abd4cd69362f0bc76920ec64bf3645271e7be4ebe487'

def fetch16(source):
    from concurrent.futures import ThreadPoolExecutor
    source.mkdir(parents=True, exist_ok=True)
    def get(url, limit, headers=None):
        with urllib.request.urlopen(urllib.request.Request(url, headers=headers or {}), timeout=60) as response:
            data = response.read(limit + 1)
            if len(data) > limit: raise ValueError('Source exceeds bounded download size')
            return data, response.headers, response.status
    data, _, _ = get(BASE + INVENTORY, 100000)
    (source / INVENTORY).write_bytes(data)
    size, chunk = 5760 * 2880 * 2, 1024 * 1024
    for product in PRODUCTS16:
        def part(start):
            end = min(size, start + chunk) - 1
            data, headers, status = get(BASE + 'meg016/' + product + '.img', end - start + 1, {'Range': f'bytes={start}-{end}'})
            if status != 206 or headers.get('Content-Range') != f'bytes {start}-{end}/{size}' or len(data) != end - start + 1:
                raise ValueError('Provider did not return the exact requested range')
            return data
        with ThreadPoolExecutor(max_workers=4) as pool:
            data = b''.join(pool.map(part, range(0, size, chunk)))
        (source / (product + '.img')).write_bytes(data)
        label, _, _ = get(BASE + 'meg016/' + product + '.xml', 100000)
        (source / (product + '.xml')).write_bytes(label)

def process16(source, destination):
    # Validate every input before writing any processed output.
    checksum_bytes = (source / INVENTORY).read_bytes()
    if hashlib.sha256(checksum_bytes).hexdigest() != INVENTORY_SHA:
        raise ValueError('Review changed provider checksum inventory before updating the dataset')
    checksums = {line.split()[1].replace(chr(92), '/'): line.split()[0].lower() for line in checksum_bytes.decode().splitlines() if len(line.split()) == 2}
    outputs, inventory = [], []
    for product, (expected, expected_label, output) in PRODUCTS16.items():
        raw = (source / (product + '.img')).read_bytes()
        label = (source / (product + '.xml')).read_bytes()
        if len(raw) != 5760 * 2880 * 2 or hashlib.sha256(raw).hexdigest() != expected:
            raise ValueError(f'{product}: incorrect size or pinned SHA256')
        if hashlib.sha256(label).hexdigest() != expected_label:
            raise ValueError(f'{product}: changed label requires review')
        for suffix, data in [('img', raw), ('xml', label)]:
            if hashlib.md5(data).hexdigest() != checksums.get('meg016/' + product + '.' + suffix):
                raise ValueError(f'{product}: provider inventory mismatch')
        root = ET.fromstring(label)
        image = root.find('.//p:Array_2D_Image', NS)
        if image is None or image.findtext('p:Element_Array/p:data_type', namespaces=NS) != 'SignedMSB2':
            raise ValueError('Unexpected MOLA type')
        axes = image.findall('p:Axis_Array', NS)
        if [(a.findtext('p:axis_name', namespaces=NS), int(a.findtext('p:elements', namespaces=NS)), int(a.findtext('p:sequence_number', namespaces=NS))) for a in axes] != [('Line', 2880, 1), ('Sample', 5760, 2)]:
            raise ValueError('Unexpected MOLA axes/order')
        if root.findtext('p:Identification_Area/p:version_id', namespaces=NS) != '1.1':
            raise ValueError('Unexpected MOLA label version')
        grid = array.array('h'); grid.frombytes(raw)
        if sys.byteorder == 'little': grid.byteswap()
        bounds, zeros = [min(grid), max(grid)], grid.count(0)
        if product.startswith('megc') and bounds[0] < 0: raise ValueError('Negative/missing observation count')
        if sys.byteorder != 'little': grid.byteswap()
        converted = grid.tobytes()
        outputs.extend([(output, converted), (product + '.xml', label)])
        inventory.append({'product': product, 'label_version': '1.1', 'source_url': BASE + 'meg016/' + product + '.img', 'source_sha256': expected, 'source_md5': checksums['meg016/' + product + '.img'], 'label_sha256': expected_label, 'output': output, 'output_sha256': hashlib.sha256(converted).hexdigest(), 'range': bounds, 'zero_bins': zeros})
    metadata = {'id': 'mgs-mola-megdr-16ppd', 'version': '2003-grid-pds4-label-1.1', 'doi': '10.17189/1z1b-kv84',
        'provider': 'NASA PDS / MGS MOLA Team / Washington University Geosciences Node', 'attribution': 'Smith, D. E., MGS MOLA MEGDR, NASA Planetary Data System',
        'rights': 'Public domain PDS archive data; see provider FAQ and source registry', 'rights_evidence': 'https://pds-geosciences.wustl.edu/documents/ViewingPDSImagesAndExportingToOtherFormats.pdf',
        'shape': [2880, 5760], 'cell_degrees': 0.0625, 'equatorial_spacing_km': 3.7044613374, 'source_reference_radius_m': 3396000,
        'grid': 'Pixel centres: 89.96875 N, 0.03125 E; north-to-south, east longitude', 'coordinate_system': 'IAU2000 planetocentric latitude, positive-east longitude',
        'vertical_datum': 'GMM3 (mgm1025), degree/order 50 areoid; topography = radius minus areoid radius', 'unit': 'm', 'counts_unit': 'observations per bin',
        'processing': 'Exact big-to-little endian conversion only; bilinear sampling at runtime', 'uncertainty_m': None,
        'coverage_limit': 'Provider reports about 55% of equatorial bins have at least one shot; profile gaps reach 12 km. Pixel spacing is not independent measured resolution. Zero-shot bins are provider-interpolated. Counts are not accuracy/confidence.',
        'observations': 'Principally February 1999–June 2001, plus polar observations; 2003 grid, September 2026 label correction does not add observations',
        'checksum_inventory_url': BASE + INVENTORY, 'checksum_inventory_sha256': INVENTORY_SHA, 'products': inventory}
    outputs.extend([(INVENTORY, checksum_bytes), ('metadata.json', (json.dumps(metadata, indent=2) + '\n').encode())])
    destination.mkdir(parents=True, exist_ok=True)
    for name, data in outputs: (destination / name).write_bytes(data)
    print(json.dumps(metadata, indent=2))

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--ppd', choices=[4, 16], type=int, default=4)
    parser.add_argument('--source-dir', type=Path)
    parser.add_argument('--output-dir', type=Path)
    parser.add_argument('--fetch', action='store_true')
    args = parser.parse_args()
    args.source_dir = args.source_dir or Path("tools/data/mola16" if args.ppd == 16 else "tools/data/mola")
    args.output_dir = args.output_dir or Path("crates/cosmogon_sim/data/mola16" if args.ppd == 16 else "crates/cosmogon_sim/data/mola")
    if args.ppd == 16:
        if args.fetch: fetch16(args.source_dir)
        process16(args.source_dir, args.output_dir)
        sys.exit(0)
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
