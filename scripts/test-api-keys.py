#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
# Run against an installation you administer; disposable fixtures are removed on exit.
import urllib.request, urllib.error, http.cookiejar, json, uuid, subprocess, hashlib
import argparse, getpass
parser = argparse.ArgumentParser(description='Integration test: creates and removes disposable users, keys, and a project. Apply migrations first.')
parser.add_argument('--base-url', default='http://127.0.0.1:18082')
parser.add_argument('--mariadb', default='mariadb', help='MariaDB client executable')
parser.add_argument('--socket', required=True)
parser.add_argument('--db-user', default=getpass.getuser())
parser.add_argument('--database', required=True)
parser.add_argument('--expected-version', help='also check the deployed /api/version value')
args = parser.parse_args()
base = args.base_url.rstrip('/')
cli = [args.mariadb, '--no-defaults', '--socket=' + args.socket, '-u', args.db_user, '-N', '-B', args.database]

def sql(q):
    return subprocess.run(cli, input=q, text=True, capture_output=True, check=True).stdout.strip()

def browser():
    return urllib.request.build_opener(urllib.request.HTTPCookieProcessor(http.cookiejar.CookieJar()), urllib.request.ProxyHandler({}))

def req(b, path, method='GET', data=None, key=None, status=200, scheme='Bearer'):
    headers = {} if data is None else {'Content-Type': 'application/json'}
    if key is not None:
        headers['Authorization'] = scheme + ' ' + key
    r = urllib.request.Request(base + path, method=method, data=None if data is None else json.dumps(data).encode(), headers=headers)
    try:
        with b.open(r) as response:
            code = response.status
            raw = response.read()
            hdr = dict(response.headers)
    except urllib.error.HTTPError as e:
        code = e.code
        raw = e.read()
        hdr = dict(e.headers)
    # Do not include bodies or credentials in failed assertion messages.
    assert code == status, (path, code, status)
    if not raw:
        body = None
    else:
        try:
            body = json.loads(raw)
        except (json.JSONDecodeError, UnicodeDecodeError):
            body = raw
    return (body, hdr)
names = ['key_check_' + uuid.uuid4().hex[:10] for _ in range(3)]
ids = []
project_name = 'api-key-check-' + uuid.uuid4().hex
fixture_hash = 'ZQCSebz8oYtDe+zogbySq+ErbFvf6gS9zAF8MV3/RBRHPdSMOBXlzO5ZaZg6tXMPWjHXxVhcC6yHNhkHPsqUGQ=='
try:
    for i, name in enumerate(names):
        sql(f"INSERT INTO FOSUser(username,username_canonical,enabled,salt,password,locked,expired,roles,credentials_expired) VALUES ('{name}','{name}',1,'fixture-salt','{fixture_hash}',0,0,'a:0:{{}}',0); INSERT INTO PartKeeprUser(username,admin,legacy,active,protected) VALUES ('{name}',{int(i == 2)},0,1,0);")
        ids.append(int(sql(f"SELECT id FROM PartKeeprUser WHERE username='{name}';")))
    sessions = [browser() for _ in names]
    for b, name in zip(sessions, names):
        req(b, '/api/login', 'POST', {'username': name, 'password': 'migration-test-password'})
    version_info, _ = req(sessions[0], '/api/version')
    if args.expected_version:
        assert version_info['version'] == args.expected_version, 'Unexpected deployed version'
    anon = browser()
    req(anon, '/api/me/api-keys', status=401)
    ro, h = req(sessions[0], '/api/me/api-keys', 'POST', {'name': 'Read report', 'access': 'read', 'expires_in_days': 90}, status=201)
    assert h.get('cache-control', h.get('Cache-Control')) == 'no-store'
    token = ro['api_key']
    kid = ro['key']['id']
    assert sql(f'SELECT key_hash FROM ApiKey WHERE id={kid};') == hashlib.sha256(token.encode()).hexdigest()
    rows, _ = req(sessions[0], '/api/me/api-keys')
    assert len(rows) == 1 and all(('api_key' not in row and 'key_hash' not in row for row in rows))
    identity, h = req(anon, '/api/me', key=token)
    assert identity['user_id'] == ids[0] and (not any((k.lower() == 'set-cookie' for k in h)))
    req(anon, '/api/parts?limit=1', key=token, scheme='bearer')
    req(anon, '/files/PartAttachment/0', method='HEAD', key=token, status=404)
    req(anon, '/api/projects', 'POST', {'name': project_name}, key=token, status=403)
    req(anon, '/api/me/api-keys', key=token, status=403)
    req(sessions[0], '/api/me', key=token[:-1] + ('a' if token[-1] != 'a' else 'b'), status=401)
    req(sessions[0], '/api/me', key='bad-key', status=401)
    req(sessions[0], '/api/me', key=token, scheme='Basic', status=401)
    assert sql(f'SELECT last_used_at IS NOT NULL FROM ApiKey WHERE id={kid};') == '1'
    other, _ = req(sessions[1], '/api/me/api-keys')
    assert other == []
    req(sessions[1], f'/api/me/api-keys/{kid}', 'PATCH', {'name': 'stolen'}, status=404)
    req(sessions[1], f'/api/me/api-keys/{kid}', 'DELETE', status=404)
    req(sessions[1], '/api/me/api-keys', 'POST', {'name': 'wrong owner', 'access': 'read', 'user_id': ids[0]}, status=403)
    req(sessions[0], f'/api/me/api-keys/{kid}', 'PATCH', {'name': 'Renamed'}, status=204)
    req(sessions[0], f'/api/me/api-keys/{kid}', 'PATCH', {'name': 'Renamed'}, status=204)
    req(sessions[0], '/api/me/api-keys', 'POST', {'name': 'bad', 'access': 'admin'}, status=400)
    req(sessions[0], '/api/me/api-keys', 'POST', {'name': 'bad', 'access': 'read', 'expires_in_days': 0}, status=400)
    rw, _ = req(sessions[0], '/api/me/api-keys', 'POST', {'name': 'Stock importer', 'access': 'read_write', 'expires_in_days': None}, status=201)
    write = rw['api_key']
    project, _ = req(anon, '/api/projects', 'POST', {'name': project_name}, key=write, status=201)
    assert int(sql(f"SELECT user_id FROM Project WHERE name='{project_name}';")) == ids[0]
    for path, method, data in [('/api/me/api-keys', 'GET', None), ('/api/me/api-key-owners', 'GET', None), ('/api/me/api-keys', 'POST', {'name': 'replacement', 'access': 'read'}), ('/api/me/profile', 'GET', None), ('/api/me/profile', 'PUT', {'email': 'key@example.com'}), ('/api/me/password', 'POST', {})]:
        req(anon, path, method, data, key=write, status=403)
    for column in ['active']:
        sql(f'UPDATE PartKeeprUser SET {column}=0 WHERE id={ids[0]};')
        req(anon, '/api/me', key=write, status=401)
        sql(f'UPDATE PartKeeprUser SET {column}=1 WHERE id={ids[0]};')
    sql(f"UPDATE FOSUser SET locked=1 WHERE username='{names[0]}';")
    req(anon, '/api/me', key=write, status=401)
    sql(f"UPDATE FOSUser SET locked=0 WHERE username='{names[0]}';")
    exp, _ = req(sessions[0], '/api/me/api-keys', 'POST', {'name': 'Expire', 'access': 'read', 'expires_in_days': 1}, status=201)
    sql(f"UPDATE ApiKey SET expires_at=UTC_TIMESTAMP()-INTERVAL 1 SECOND WHERE id={exp['key']['id']};")
    req(anon, '/api/me', key=exp['api_key'], status=401)
    req(sessions[0], f'/api/me/api-keys/{kid}', 'DELETE', status=204)
    req(sessions[0], f'/api/me/api-keys/{kid}', 'DELETE', status=204)
    req(anon, '/api/me', key=token, status=401)
    req(sessions[0], '/api/me')
    owners, _ = req(sessions[0], '/api/me/api-key-owners')
    assert [o['user_id'] for o in owners] == [ids[0]]
    assigned, _ = req(sessions[2], '/api/me/api-keys', 'POST', {'name': 'Assigned by admin', 'access': 'read', 'user_id': ids[1]}, status=201)
    assigned_identity, _ = req(anon, '/api/me', key=assigned['api_key'])
    assert assigned_identity['user_id'] == ids[1]
    admin_rows, _ = req(sessions[2], '/api/me/api-keys')
    assert kid in [r['id'] for r in admin_rows]
    req(sessions[2], f"/api/me/api-keys/{assigned['key']['id']}", 'DELETE', status=204)
    req(anon, '/api/me', key=assigned['api_key'], status=401)
    print('PASS: hash-only storage, one-time secret, bearer/cookie parallel auth, read-only and write attribution, owner isolation, admin assignment, expiry, revocation, disabled accounts, last-used tracking, and session-only account/key management.')
finally:
    sql(f"DELETE FROM Project WHERE name='{project_name}';")
    for name in names:
        sql(f"DELETE FROM UserPreference WHERE user_id IN (SELECT id FROM PartKeeprUser WHERE username='{name}'); DELETE FROM PartKeeprUser WHERE username='{name}'; DELETE FROM FOSUser WHERE username='{name}';")
    print('Temporary users, keys, and project removed.')
