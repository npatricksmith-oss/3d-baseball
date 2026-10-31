"""Independent analytic and spec checks.

Run either:
    python3 -m unittest analysis/oracle/test_oracle.py
    python3 analysis/oracle/test_oracle.py
"""
import math
import unittest

try:
    from . import bbphys as p
    from . import gen_golden as golden
except ImportError:
    import bbphys as p
    import gen_golden as golden


class OracleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.corpus = golden.build_corpus()
        cls.flights = {c['name']: c for c in cls.corpus['flight']}
        cls.collisions = {c['name']: c for c in cls.corpus['collision']}

    def assertVectorClose(self, actual, expected, tolerance=1e-9):
        for a, e in zip(actual, expected):
            self.assertAlmostEqual(a, e, delta=tolerance*max(1.0, abs(e)))

    def test_units_and_ball(self):
        self.assertEqual(p.MPH*100, 44.704)
        self.assertAlmostEqual(p.RADIUS*2*math.pi, .231775, delta=1e-16)
        self.assertAlmostEqual(p.AREA, math.pi*p.RADIUS**2, delta=1e-18)
        self.assertAlmostEqual(p.BALL_INERTIA, .4*.14529*p.RADIUS**2, delta=1e-18)
        self.assertAlmostEqual(p.FT, 12*p.INCH, delta=1e-16)
        self.assertAlmostEqual(p.RPM*60, 2*math.pi, delta=1e-15)
        self.assertEqual(p.OZ, .028349523)

    def test_air_density(self):
        self.assertAlmostEqual(p.air_density(), 1.1938, delta=5e-5)
        self.assertLess(p.air_density(elev_ft=5280), p.air_density())
        self.assertLess(p.air_density(temp_f=100), p.air_density())
        self.assertGreater(p.air_density(temp_f=25), p.air_density())
        self.assertLess(p.air_density(rh_percent=100), p.air_density(rh_percent=0))

    def test_A1_A2(self):
        a1, a2 = (self.flights[name]['result'] for name in ('A1', 'A2'))
        self.assertAlmostEqual(a1['r'][1]/p.FT, 406, delta=3)
        self.assertAlmostEqual(a1['t'], 5.6, delta=.1)
        self.assertLess(abs(a1['r'][1]-a2['r'][1])/p.FT, 1)
        self.assertLess(p.norm(a2['w']), p.norm(a1['w']))

    def test_A3_step_convergence(self):
        c = self.flights['A1']
        state = p.State(c['r0'], c['v0'], c['w0'])
        coarse = p.integrate(state, c['rho'], dt=.01)
        medium = p.integrate(state, c['rho'], dt=.001)
        self.assertLess(abs(coarse['r'][1]-medium['r'][1])/p.FT, .01)
        self.assertVectorClose(medium['r'], c['result']['r'], tolerance=1e-9)
        self.assertAlmostEqual(medium['apex_z'], c['result']['apex_z'], delta=1e-8)

    def test_A4_vacuum_analytic(self):
        c = self.flights['A4_vacuum']
        r0, v0, result = c['r0'], c['v0'], c['result']
        exact_t = (v0[2]+math.sqrt(v0[2]**2+2*p.G*r0[2]))/p.G
        self.assertAlmostEqual(result['t']/exact_t, 1.0, delta=1e-9)
        expected_r = (r0[0]+v0[0]*exact_t, r0[1]+v0[1]*exact_t, 0.0)
        self.assertVectorClose(result['r'], expected_r, tolerance=1e-9)
        self.assertVectorClose(result['v'], (v0[0], v0[1], v0[2]-p.G*exact_t), tolerance=1e-9)
        self.assertVectorClose(result['w'], c['w0'], tolerance=1e-14)
        exact_apex = r0[2]+v0[2]**2/(2*p.G)
        self.assertAlmostEqual(result['apex_z']/exact_apex, 1.0, delta=1e-9)

    def test_continuous_spin_decay(self):
        # Check the ODE coefficient and preservation of spin direction.
        s = (0, 0, 1, 0, 44.704, 0, 20, -30, 40)
        rhs = p.flight_rhs(s, 0, 20)
        self.assertVectorClose(rhs[6:], (-1, 1.5, -2), tolerance=1e-14)
        result = self.flights['batted_tilted_gyro']['result']
        w0 = self.flights['batted_tilted_gyro']['w0']
        ratios = [a/b for a, b in zip(result['w'], w0)]
        self.assertAlmostEqual(ratios[0], ratios[1], delta=1e-13)
        self.assertAlmostEqual(ratios[1], ratios[2], delta=1e-13)
        self.assertGreater(ratios[0], 0)
        self.assertLess(ratios[0], 1)
        # In a vertical vacuum flight, integral |v| dt is analytic, including
        # the reversal at the apex. Choose v_z=g so that apex is on a step edge.
        initial = p.State((0, 0, 1), (0, 0, p.G), (20, -30, 40))
        vacuum = p.integrate(initial, 0, tau100=20)
        distance = p.G/2 + (p.G*vacuum['t']-p.G)**2/(2*p.G)
        expected = p.scale(initial.w, math.exp(-distance/(20*44.704)))
        self.assertVectorClose(vacuum['w'], expected, tolerance=1e-10)

    def test_A5_magnus_signs(self):
        for velocity, spin, sign in [((0, 40, 0), (200, 0, 0), 1),
                                     ((0, -40, 0), (-200, 0, 0), 1),
                                     ((0, -40, 0), (200, 0, 0), -1)]:
            with self.subTest(velocity=velocity, spin=spin):
                full = p.acceleration(velocity, spin, p.air_density())
                drag_gravity = p.acceleration(velocity, spin, p.air_density(), magnus=False)
                lift = p.sub(full, drag_gravity)
                self.assertGreater(sign*lift[2], 0)
                self.assertAlmostEqual(lift[0], 0, delta=1e-14)
                self.assertAlmostEqual(lift[1], 0, delta=1e-14)
        # Pure gyro adds spin-dependent drag but no lift.
        self.assertVectorClose(p.acceleration((0, -40, 0), (0, -200, 0), 1.2),
                               p.acceleration((0, -40, 0), (0, -200, 0), 1.2, magnus=False))
        self.assertEqual(p.acceleration(p.ZERO, (100, 20, 0), 1.2), (0, 0, -p.G))

    def test_air_relative_velocity(self):
        v, wind, w = (10, -40, 3), (4, 2, -1), (-200, 50, 70)
        self.assertVectorClose(p.acceleration(v, w, 1.2, wind=wind),
                               p.acceleration(p.sub(v, wind), w, 1.2), tolerance=1e-14)

    def test_A6_mirror(self):
        plus, minus = (self.flights[n]['result'] for n in ('A6_mirror_plus', 'A6_mirror_minus'))
        self.assertEqual(plus['t'], minus['t'])
        self.assertVectorClose(minus['r'], (-plus['r'][0], plus['r'][1], plus['r'][2]), tolerance=1e-12)
        self.assertVectorClose(minus['v'], (-plus['v'][0], plus['v'][1], plus['v'][2]), tolerance=1e-12)
        self.assertVectorClose(minus['w'], (plus['w'][0], -plus['w'][1], -plus['w'][2]), tolerance=1e-12)
        self.assertEqual(plus['apex_z'], minus['apex_z'])

    def test_event_locations(self):
        for c in self.corpus['flight']:
            with self.subTest(name=c['name']):
                axis = 2 if c['stop']['kind'] == 'ground' else 1
                target = 0 if axis == 2 else c['stop']['y']
                # Bisection time uncertainty <=5e-11 s, plus roundoff.
                self.assertAlmostEqual(c['result']['r'][axis], target, delta=3e-9)
                self.assertLess(c['result']['v'][axis], 0)
                self.assertGreaterEqual(c['result']['apex_z'], c['r0'][2])
                self.assertGreaterEqual(c['result']['apex_z'], c['result']['r'][2])
        initial = p.State((0, 0, 1), (0, -10, 0), p.ZERO)
        result = p.integrate(initial, 0, stop=dict(kind='plane_y', y=-2), dt=.07)
        self.assertAlmostEqual(result['t'], .2, delta=5e-11)
        self.assertAlmostEqual(result['r'][2], 1-.5*p.G*.2**2, delta=1e-10)
        # Ascending launch and descending ground crossing both occur in step 1.
        tiny = p.integrate(p.State(p.ZERO, (0, 1, 1e-4), p.ZERO), 0)
        self.assertAlmostEqual(tiny['t'], 2e-4/p.G, delta=1e-10)

    def test_A7_head_on_q(self):
        bat = self.corpus['bat_default']
        for c in self.corpus['collision']:
            if not c['name'].startswith('A7_'):
                continue
            with self.subTest(name=c['name']):
                b = c['r2'][0]
                ratio = c['m']*(1/bat['M']+b*b/bat['I_cm'])
                q = (c['e_n']-ratio)/(1+ratio)
                expected = q*(-c['v'][1])+(1+q)*c['V'][1]
                self.assertAlmostEqual(c['result']['v'][1], expected, delta=1e-9)
                self.assertVectorClose(c['result']['w'], p.ZERO)
        sweet = self.collisions['A8_sweet_spot_q020']
        self.assertAlmostEqual(sweet['result']['v'][1]/p.MPH, 107, delta=1e-10)

    def test_A9_gripping_relation(self):
        c = self.collisions['A9_stationary_ball_oblique']
        self.assertEqual(c['result']['branch'], 'stick')
        bt = c['inv_M2'] + c['r2'][0]**2*self.corpus['bat_default']['I_cm']**-1
        rx = c['m']*p.ALPHA/(1+p.ALPHA)*bt
        expected_spin = (1+c['e_t'])*c['V'][2]/(p.RADIUS*(1+rx)*(1+p.ALPHA))
        self.assertAlmostEqual(-c['result']['w'][0], expected_spin, delta=1e-10)
        self.assertAlmostEqual(c['result']['v'][2], p.ALPHA*p.RADIUS*expected_spin, delta=1e-11)

    def test_collision_energy_and_conservation(self):
        for c in self.corpus['collision']:
            with self.subTest(name=c['name']):
                r = c['result']
                before = p.kinetic_energy(c['m'], c['inv_I1'], c['v'], c['w'], c['inv_M2'], c['inv_I2'], c['V'], c['W'])
                after = p.kinetic_energy(c['m'], c['inv_I1'], r['v'], r['w'], c['inv_M2'], c['inv_I2'], r['V'], r['W'])
                self.assertLessEqual(after, before+64*math.ulp(max(1.0, before)))
                self.assertVectorClose(p.scale(p.sub(r['v'], c['v']), c['m']), r['J'], tolerance=2e-13)
                self.assertVectorClose(p.sub(r['w'], c['w']),
                                       p.matvec(c['inv_I1'], p.cross(c['r1'], r['J'])), tolerance=2e-13)
                if c['inv_M2']:
                    self.assertVectorClose(p.scale(p.sub(r['V'], c['V']), 1/c['inv_M2']), p.scale(r['J'], -1), tolerance=2e-13)
                    self.assertVectorClose(p.sub(r['W'], c['W']),
                                           p.scale(p.matvec(c['inv_I2'], p.cross(c['r2'], r['J'])), -1), tolerance=2e-13)
                else:
                    self.assertEqual(r['V'], c['V'])
                    self.assertEqual(r['W'], c['W'])
        self.assertTrue(self.collisions['energy_guard_stress']['result']['energy_guard'])
        stress = self.collisions['energy_guard_stress']
        # Independently rebuild the rejected first attempt and verify that it
        # actually gains energy before the prescribed retry.
        r1, r2 = p.skew(stress['r1']), p.skew(stress['r2'])
        a = p.matmul(p.matmul(r1, stress['inv_I1']), r1)
        b = p.matmul(p.matmul(r2, stress['inv_I2']), r2)
        K = tuple(tuple((1/stress['m']+stress['inv_M2'] if i == j else 0)-a[i][j]-b[i][j]
                        for j in range(3)) for i in range(3))
        old = stress['v']  # Both bodies have zero angular velocity, body 2 rests.
        cn = p.dot(old, stress['n'])
        ct = p.sub(old, p.scale(stress['n'], cn))
        target = p.add(p.scale(stress['n'], -stress['e_n']*cn), p.scale(ct, -stress['e_t']))
        rejected = p.solve(K, p.sub(target, old))
        jn = p.dot(rejected, stress['n'])
        if p.norm(p.sub(rejected, p.scale(stress['n'], jn))) > stress['mu']*jn:
            direction = p.sub(stress['n'], p.scale(p.unit(ct), stress['mu']))
            rejected = p.scale(direction, -(1+stress['e_n'])*cn/p.dot(stress['n'], p.matvec(K, direction)))
        energy_change = p.dot(old, rejected)+.5*p.dot(rejected, p.matvec(K, rejected))
        self.assertGreater(energy_change, 0)
        retry = p.collision(**dict({k: v for k, v in stress.items() if k not in ('name', 'result')}, e_t=0))
        self.assertVectorClose(retry['J'], stress['result']['J'], tolerance=1e-14)

    def test_collision_restitution_and_friction(self):
        branches = set()
        for c in self.corpus['collision']:
            with self.subTest(name=c['name']):
                r = c['result']
                branches.add(r['branch'])
                old = p.sub(p.add(c['v'], p.cross(c['w'], c['r1'])), p.add(c['V'], p.cross(c['W'], c['r2'])))
                new = p.sub(p.add(r['v'], p.cross(r['w'], c['r1'])), p.add(r['V'], p.cross(r['W'], c['r2'])))
                cn = p.dot(old, c['n'])
                self.assertAlmostEqual(p.dot(new, c['n']), -c['e_n']*cn, delta=1e-10)
                jn = p.dot(r['J'], c['n'])
                jt = p.sub(r['J'], p.scale(c['n'], jn))
                self.assertGreater(jn, 0)
                if r['branch'] == 'stick':
                    self.assertLessEqual(p.norm(jt), c['mu']*jn+1e-12)
                    et = 0 if r['energy_guard'] else c['e_t']
                    self.assertVectorClose(p.sub(new, p.scale(c['n'], p.dot(new, c['n']))),
                                           p.scale(p.sub(old, p.scale(c['n'], cn)), -et), tolerance=1e-10)
                else:
                    self.assertAlmostEqual(p.norm(jt), c['mu']*jn, delta=1e-12)
                    self.assertLess(p.dot(jt, old), 0)
        self.assertEqual(branches, {'stick', 'slide'})

    def test_bat_profile_and_quadrature(self):
        for station, diameter in [(0, 1.35), (.5, 1.35), (.75, 1.14),
                                   (1, .93), (11, .93), (22, 2.5), (34, 2.5)]:
            self.assertAlmostEqual(2*p.bat_radius(station*p.INCH)/p.INCH, diameter, delta=1e-13)
        # Independent closed-form integrals for the piecewise polynomial/power profile.
        # d = a + b*t**exponent on each segment, t in [0,1].
        mass = first = second = radial = 0.0
        for low, high, a, b, exponent in [(0, .5, 1.35, 0, 1), (.5, 1, 1.35, -.42, 1),
                                          (1, 11, .93, 0, 1), (11, 22, .93, 1.57, 1.6),
                                          (22, 34, 2.5, 0, 1)]:
            length, start = (high-low)*p.INCH, low*p.INCH
            density_factor = 660*math.pi*(p.INCH/2)**2*length
            for k in range(3):
                coeff = math.comb(2, k)*a**(2-k)*b**k
                power = exponent*k
                mass += density_factor*coeff/(power+1)
                first += density_factor*coeff*(start/(power+1)+length/(power+2))
                second += density_factor*coeff*(start**2/(power+1)+2*start*length/(power+2)+length**2/(power+3))
            for k in range(5):
                radial += 660*math.pi*(p.INCH/2)**4*length*math.comb(4, k)*a**(4-k)*b**k/(exponent*k+1)
        cm = first/mass
        icm = second-mass*cm**2+radial/4
        expected = dict(M=mass, u_cm=cm, I_cm=icm, I_ax=radial/2,
                        moi_6in=icm+mass*(cm-6*p.INCH)**2, L=34*p.INCH)
        for key, value in expected.items():
            self.assertAlmostEqual(self.corpus['bat_default'][key], value, delta=3e-12)

    def test_world_inverse_inertia(self):
        b = self.corpus['bat_default']
        axis = p.unit((1, 2, 3))
        inv = p.bat_inverse_inertia(axis, b['I_cm'], b['I_ax'])
        perpendicular = p.unit(p.cross(axis, (1, 0, 0)))
        self.assertVectorClose(p.matvec(inv, axis), p.scale(axis, 1/b['I_ax']), tolerance=1e-13)
        self.assertVectorClose(p.matvec(inv, perpendicular), p.scale(perpendicular, 1/b['I_cm']), tolerance=1e-12)

    def test_spin_axis_four_checks(self):
        for w, axis, tilt in [((-1, 0, 0), 180, 0), ((1, 0, 0), 0, 6),
                               ((-math.sqrt(3)/2, 0, -.5), 210, 1),
                               ((-math.sqrt(3)/2, 0, .5), 150, 11)]:
            metrics = p.spin_metrics(w, (0, -40, 0))
            self.assertAlmostEqual(metrics['spin_axis_deg'], axis, delta=1e-12)
            self.assertAlmostEqual(metrics['tilt_hours'], tilt, delta=1e-12)
            self.assertAlmostEqual(metrics['efficiency'], 1, delta=1e-14)

    def test_spin_round_trip(self):
        for tilt in (0, .25, 1, 3, 6, 9, 11.75):
            for efficiency in (.1, .65, .9, 1):
                for sign in (-1, 1):
                    w = p.spin_from_tilt(tilt, efficiency, 2500, sign)
                    metrics = p.spin_metrics(w, (0, -40, 0))
                    self.assertAlmostEqual(metrics['efficiency'], efficiency, delta=1e-14)
                    self.assertAlmostEqual((metrics['tilt_hours']-tilt+6)%12-6, 0, delta=1e-13)
                    self.assertAlmostEqual(p.norm(w), 2500*p.RPM, delta=1e-12)
        for w in (p.ZERO, (0, -100, 0)):
            metrics = p.spin_metrics(w, (0, -40, 0))
            self.assertEqual(metrics['efficiency'], 0)
            self.assertIsNone(metrics['spin_axis_deg'])
            self.assertIsNone(metrics['tilt_hours'])

    def test_strike_rounded_corners_and_telemetry(self):
        height = 1.8
        top = .535*height
        self.assertTrue(p.plate_strike((0, p.PLATE_Y, .8), height))
        self.assertTrue(p.plate_strike((.2159+p.RADIUS*.99, p.PLATE_Y, .8), height))
        self.assertFalse(p.plate_strike((.2159+.8*p.RADIUS, p.PLATE_Y, top+.8*p.RADIUS), height))
        self.assertTrue(p.plate_strike((.2159+.6*p.RADIUS, p.PLATE_Y, top+.6*p.RADIUS), height))
        telem = p.velocity_telemetry(p.launch_velocity(100, 30, -15))
        self.assertAlmostEqual(telem['speed']/p.MPH, 100, delta=1e-12)
        self.assertAlmostEqual(telem['launch_angle_deg'], 30, delta=1e-12)
        self.assertAlmostEqual(telem['spray_angle_deg'], -15, delta=1e-12)

    def test_corpus_coverage(self):
        self.assertGreaterEqual(len(self.corpus['air_density']), 5)
        self.assertGreaterEqual(len(self.corpus['flight']), 21)
        self.assertGreaterEqual(len(self.corpus['collision']), 12)
        for name, cases in self.corpus.items():
            if isinstance(cases, list) and cases and 'name' in cases[0]:
                self.assertEqual(len(cases), len({c['name'] for c in cases}))


if __name__ == '__main__':
    unittest.main()
