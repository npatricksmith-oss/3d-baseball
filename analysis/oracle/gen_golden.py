"""Generate reproducible SI golden fixtures; run from any working directory."""
import json
import math
from pathlib import Path

try:
    from . import bbphys as p
except ImportError:
    import bbphys as p

HERE = Path(__file__).resolve().parent


def air_cases():
    inputs = [(70, 29.92, 50, 15), (100, 29.70, 20, 15),
              (25, 30.20, 30, 15), (70, 29.92, 50, 5280),
              (85, 29.92, 100, 15), (70, 29.92, 0, 0)]
    return [dict(temp_f=t, pressure_inhg=pr, rh_pct=rh, elev_ft=h,
                 rho=p.air_density(t, pr, rh, h)) for t, pr, rh, h in inputs]


def flight_inputs():
    rho = p.air_density()
    cases = []
    def case(name, r, v, w, tau=None, stop=None, density=rho):
        cases.append(dict(name=name, r0=r, v0=v, w0=w, rho=density, tau100=tau,
                          stop={'kind': 'ground'} if stop is None else stop))
    r = (0.0, 0.0, 3*p.FT)
    v = p.launch_velocity(100, 30)
    w = (2500*p.RPM, 0.0, 0.0)
    case('A1', r, v, w)
    case('A2', r, v, w, tau=20.0)
    case('A4_vacuum', (1.2, -2.3, 3*p.FT), (7.0, 31.0, 18.0),
         (123.0, -67.0, 89.0), density=0.0)
    pitches = [('four_seam_95_2300', 95, 2300, 1.0, False),
               ('four_seam_95_2300_eff90', 95, 2300, 0.9, False),
               ('curve_80_2600', 80, 2600, 1.0, True),
               ('change_85_1700', 85, 1700, 1.0, False)]
    for name, speed, rpm, efficiency, top in pitches:
        spin = rpm*p.RPM
        w = (spin if top else -spin*efficiency,
             -spin*math.sqrt(1-efficiency**2), 0.0)
        case(name, (0.0, 54*p.FT, 6*p.FT), (0.0, -speed*p.MPH, 0.0), w,
             stop=dict(kind='plane_y', y=p.PLATE_Y))
    for speed, angle, rpm in [(100, 28, 2200), (105, 28, 2400),
                              (95, 15, 1500), (90, 45, 3500)]:
        case(f'batted_{speed}_{angle}_{rpm}', r, p.launch_velocity(speed, angle),
             (rpm*p.RPM, 0.0, 0.0))
    for name, r0, v0, w0, tau in [
        ('pitch_tilted_gyro', (-.6, 16.8, 1.95), (1.2, -43, -.8), (-170, 90, -85), 22.0),
        ('pitch_slider', (.5, 16.2, 1.8), (-1.1, -38, -1.5), (65, 180, 155), None),
        ('pitch_sidespin', (.2, 16.7, 1.9), (.4, -36, .2), (0, 0, 210), 20.0),
        ('pitch_gyro', (-.3, 16.5, 1.85), (1.0, -40, -.5), (6, -240, -3), 25.0),
    ]:
        case(name, r0, v0, w0, tau=tau, stop=dict(kind='plane_y', y=p.PLATE_Y))
    for name, r0, v0, w0, tau in [
        ('batted_tilted_gyro', (.7, 1.0, .95), p.launch_velocity(102, 24, 23), (180, 85, -60), 22.0),
        ('batted_topspin', (-.5, .4, .8), p.launch_velocity(93, 10, -18), (-160, 65, 95), None),
        ('batted_sidespin', (.3, -.2, 1.05), p.launch_velocity(88, 32, 8), (0, 0, 260), 20.0),
        ('batted_high_gyro', (-.8, .6, 1.1), (8, 30, 19), (55, 230, 100), 25.0),
    ]:
        case(name, r0, v0, w0, tau=tau)
    case('A6_mirror_plus', (1.2, 2.0, 3*p.FT), (5.0, 35.0, 21.0), (220, 50, 100), tau=22.0)
    case('A6_mirror_minus', (-1.2, 2.0, 3*p.FT), (-5.0, 35.0, 21.0), (220, -50, -100), tau=22.0)
    return cases


def collision_inputs():
    bat = p.bat_mass_properties()
    inv1 = p.diagonal(1/p.BALL_INERTIA)
    inv2 = p.bat_inverse_inertia((1, 0, 0), bat['I_cm'], bat['I_ax'])
    cases = []
    def case(name, **changes):
        data = dict(name=name, m=p.MASS, inv_I1=inv1, v=(0, -85*p.MPH, 0), w=p.ZERO,
                    r1=(0, -p.RADIUS, 0), inv_M2=1/bat['M'], inv_I2=inv2,
                    V=(0, 75*p.MPH, 0), W=p.ZERO, r2=(0, 0, 0), n=(0, 1, 0),
                    e_n=.45, e_t=.4, mu=.5)
        data.update(changes)
        cases.append(data)
    for station in (6, 22, 26, 28, 30, 32.5):
        case(f'A7_head_on_{station:g}in', r2=(station*p.INCH-bat['u_cm'], 0, 0))
    ss = bat['L']-.15
    ratio = p.MASS*(1/bat['M'] + (ss-bat['u_cm'])**2/bat['I_cm'])
    case('A8_sweet_spot_q020', r2=(ss-bat['u_cm'], 0, 0), e_n=.2+1.2*ratio)
    case('oblique_stick', v=(3, -35, -2), w=(-40, 12, 20),
         r2=(.12, p.bat_radius(28*p.INCH), 0), mu=.8)
    case('oblique_stick_rotating_bat', v=(-2, -39, 1), w=(20, -30, 15),
         V=(1, 18, 0), W=(0, 0, 35), r2=(.13, .03, 0), mu=.8)
    psi = math.radians(20)
    case('A9_stationary_ball_oblique', v=p.ZERO, V=(0, 30*math.cos(psi), 30*math.sin(psi)),
         r2=(.1, 0, 0), mu=1.0)
    case('oblique_slide_low_mu', v=(24, -35, -18), w=(50, 20, 90), r2=(.15, .03, 0), mu=.03)
    case('oblique_slide_tilted', v=(-25, -32, 20), w=(80, -25, 10),
         n=(0, .8, .6), r1=(0, -.8*p.RADIUS, -.6*p.RADIUS), r2=(.2, .024, .018), mu=.02)
    for name, v, w, en, mu in [
        ('ground_vertical', (0, 0, -12), p.ZERO, .45, .45),
        ('ground_stick', (2, -1, -10), (4, -10, 0), .50, .55),
        ('ground_slide', (25, 10, -3), (0, 0, 30), .42, .45),
        ('ground_spinning', (-8, 3, -8), (150, -200, 20), .55, .4),
    ]:
        case(name, v=v, w=w, r1=(0, 0, -p.RADIUS), inv_M2=0.0, inv_I2=p.ZERO_MATRIX,
             V=p.ZERO, W=p.ZERO, r2=p.ZERO, n=(0, 0, 1), e_n=en, e_t=0.0, mu=mu)
    case('energy_guard_stress', v=(80, -20, 0), V=p.ZERO, r2=(.3, .03, .2),
         e_n=.8, e_t=.9, mu=20.0)
    # Non-diagonal world inertia: rotate the bat's axis into all three axes.
    case('oblique_rotated_axis', inv_I2=p.bat_inverse_inertia((1, 2, 3), bat['I_cm'], bat['I_ax']),
         r2=(.08, .12, .16), v=(2, -35, 1), mu=1.0)
    return cases


def spin_inputs():
    result = []
    for tilt, eff, rate, sign in [(0, 1, 2300, 1), (6, 1, 2600, 1),
                                   (1, 1, 2300, 1), (11, 1, 2300, 1),
                                   (1, .9, 2300, 1), (11, .65, 2100, -1),
                                   (3.5, .35, 2500, 1)]:
        result.append(dict(w=p.spin_from_tilt(tilt, eff, rate, sign), v=(0, -40, 0)))
    result.append(dict(w=(100, 120, -40), v=(2, -39, -1)))
    return result


def build_corpus(progress=None):
    flights = flight_inputs()
    for c in flights:
        if progress:
            progress(c['name'])
        c['result'] = p.integrate(p.State(tuple(c['r0']), tuple(c['v0']), tuple(c['w0'])),
                                  c['rho'], c['tau100'], c['stop'])
    collisions = collision_inputs()
    for c in collisions:
        c['result'] = p.collision(**{k: v for k, v in c.items() if k != 'name'})
    spins = [dict(**c, **p.spin_metrics(c['w'], c['v'])) for c in spin_inputs()]
    return {'air_density': air_cases(), 'flight': flights, 'bat_default': p.bat_mass_properties(),
            'collision': collisions, 'spin_axis': spins}


def results_notes(corpus):
    cases = {c['name']: c for c in corpus['flight']}
    lines = ['## Computed results', '',
             'Generated by `python3 analysis/oracle/gen_golden.py` at dt = 0.0001 s.', '',
             f"Default air density: {p.air_density():.15g} kg/m³.", '',
             '| Case | Carry (ft) | Time (s) | Apex (ft) |',
             '| :--- | ---: | ---: | ---: |']
    for name in ('A1', 'A2', 'batted_100_28_2200', 'batted_105_28_2400',
                 'batted_95_15_1500', 'batted_90_45_3500'):
        r = cases[name]['result']
        lines.append(f"| {name} | {math.hypot(r['r'][0], r['r'][1])/p.FT:.9f} | {r['t']:.9f} | {r['apex_z']/p.FT:.9f} |")
    lines += ['', 'Pitch results at y = 0.2159 m; movement is relative to a full-flight',
              'no-Magnus twin, retaining the same total-spin drag and decay settings.', '',
              '| Pitch | Time (s) | Plate speed (mph) | IVB (in) | Horizontal (in) |',
              '| :--- | ---: | ---: | ---: | ---: |']
    for name in ('four_seam_95_2300', 'four_seam_95_2300_eff90', 'curve_80_2600', 'change_85_1700'):
        c = cases[name]
        twin = p.integrate(p.State(tuple(c['r0']), tuple(c['v0']), tuple(c['w0'])),
                           c['rho'], c['tau100'], c['stop'], magnus=False)
        r = c['result']
        lines.append(f"| {name} | {r['t']:.9f} | {p.norm(r['v'])/p.MPH:.9f} | {(r['r'][2]-twin['r'][2])/p.INCH:+.9f} | {(r['r'][0]-twin['r'][0])/p.INCH:+.9f} |")
    lines += ['', 'Computed flat-end bat:', '',
              '| Quantity | SI value |', '| :--- | ---: |']
    lines += [f'| {key} | {value:.15g} |' for key, value in corpus['bat_default'].items()]
    guard = [c['name'] for c in corpus['collision'] if c['result']['energy_guard']]
    branches = {branch: sum(c['result']['branch'] == branch for c in corpus['collision'])
                for branch in ('stick', 'slide')}
    lines += ['', f"Corpus: {len(corpus['air_density'])} air-density, {len(corpus['flight'])} flight, "
              f"{len(corpus['collision'])} collision ({branches['stick']} stick, {branches['slide']} slide), "
              f"and {len(corpus['spin_axis'])} spin-axis cases.", '',
              'Energy guard fired for: ' + ', '.join(guard) + '.', '',
              'A1 and A2 satisfy their stated bands. The §14.3 batted-ball points differ',
              'by about 2 ft from the rounded listed carries; the spec’s A1 cross-check',
              'explicitly used y0 = 2 ft, whereas these fixtures use the requested y0 = 0.',
              'The four §14.3 pitch cross-checks are not reproduced exactly: flight times',
              'are 0.00585–0.00632 s longer, plate speeds are 0.296–0.336 mph lower,',
              'and vertical-movement magnitudes are 0.391–0.448 in larger. The original',
              'aim and termination choices are not fully specified; these fixtures retain',
              'the task’s level releases and midpoint crossing. Release-to-midpoint',
              'travel is 53.2916667 ft rather than a full 54 ft.', '',
              'The bat rounds close to §6.2 (the quoted I_cm and 6-in MOI differ by',
              'about 0.38% and 0.27%). Its mass, 0.96447 kg, exceeds the stated 0.85–0.95 kg',
              'validation band; this is already inconsistent with the spec’s own 0.965 kg default.',
              'No coefficients, geometry, or golden outputs have been adjusted to force those checks.', '']
    return '\n'.join(lines)


def main():
    corpus = build_corpus(lambda name: print('Integrating', name, flush=True))
    folder = HERE / 'golden'
    folder.mkdir(exist_ok=True)
    for name, data in corpus.items():
        (folder / f'{name}.json').write_text(json.dumps(data, indent=2, allow_nan=False) + '\n')
    notes = HERE / 'NOTES.md'
    marker = '<!-- GENERATED RESULTS -->'
    text = notes.read_text().split(marker)[0].rstrip() if notes.exists() else '# Oracle notes'
    notes.write_text(text + '\n\n' + marker + '\n\n' + results_notes(corpus))
    print('Wrote five golden JSON files and refreshed NOTES.md results.', flush=True)


if __name__ == '__main__':
    main()
