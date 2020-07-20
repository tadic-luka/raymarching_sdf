#define MAX_MARCHING_STEPS 200
#define EPSILON 0.001
#define PI 3.14159265
#define FOV PI * 0.25
#define MIN_DIST 0.0
#define MAX_DIST 100.0

out vec4 FragColor;
uniform float time;
uniform vec3 camera_eye;
uniform vec2 resolution;
uniform float shininess;
uniform int obj;

float boxSDF(vec3 p) {
	vec3 b = vec3(0.5);
	vec3 q = abs(p) - b;
  return length(max(q,0.0)) + min(max(q.x,max(q.y,q.z)),0.0);
}

float roundedBoxSDF(vec3 p) {
	return boxSDF(p) - 0.3;
}

float sphereSDF(vec3 p) {
	return length(p) - 1.0;
}

float torusSDF(vec3 p) {
	vec3 t = vec3(0.3);
  vec2 q = vec2(length(p.xz)-t.x,p.y);
  return length(q)-t.y;
}

float sceneSDF(vec3 p) {
	switch (obj) {
		case 0:
			return roundedBoxSDF(p);
		case 1:
			return torusSDF(p);
		case 2:
		default:
			return sphereSDF(p);

	}
	return sphereSDF(p);
}

float shortestDistanceToSurface(vec3 eye, vec3 marchingDirection, float start, float end) {
	float depth = start;
	for (int i = 0; i < MAX_MARCHING_STEPS; i++) {
		float dist = sceneSDF(eye + depth * marchingDirection);
		if (dist < EPSILON) {
			return depth;
		}
		depth += dist;
		if (depth >= end) {
			return end;
		}
	}
	return end;
}

vec3 estimateNormal(vec3 p) {
	return normalize(vec3(
				sceneSDF(vec3(p.x + EPSILON, p.y, p.z)) - sceneSDF(vec3(p.x - EPSILON, p.y, p.z)),
				sceneSDF(vec3(p.x, p.y + EPSILON, p.z)) - sceneSDF(vec3(p.x, p.y - EPSILON, p.z)),
				sceneSDF(vec3(p.x, p.y, p.z  + EPSILON)) - sceneSDF(vec3(p.x, p.y, p.z - EPSILON))
				));
}

vec3 rayDirection(float fieldOfView, vec2 size, vec2 fragCoord) {
	vec2 xy = fragCoord - size * 0.5;
	float z = size.y / tan(fieldOfView * 0.5);
	return normalize(vec3(xy, -z));
}

/**
 * Lighting contribution of a single point light source via Phong illumination.
 *
 * The vec3 returned is the RGB color of the light's contribution.
 *
 * k_a: Ambient color
 * k_d: Diffuse color
 * k_s: Specular color
 * alpha: Shininess coefficient
 * p: position of point being lit
 * eye: the position of the camera
 * lightPos: the position of the light
 * lightIntensity: color/intensity of the light
 *
 * See https://en.wikipedia.org/wiki/Phong_reflection_model#Description
 */
vec3 phongContribForLight(vec3 k_d, vec3 k_s, float alpha, vec3 p, vec3 eye,
		vec3 lightPos, vec3 lightIntensity) {
	vec3 N = estimateNormal(p);
	vec3 L = normalize(lightPos - p);
	vec3 V = normalize(eye - p);
	vec3 R = normalize(reflect(-L, N));

	float dotLN = dot(L, N);
	float dotRV = dot(R, V);

	if (dotLN < 0.0) {
		// Light not visible from this point on the surface
		return vec3(0.0, 0.0, 0.0);
	}

	if (dotRV < 0.0) {
		// Light reflection in opposite direction as viewer, apply only diffuse
		// component
		return lightIntensity * (k_d * dotLN);
	}
	return lightIntensity * (k_d * dotLN + k_s * pow(dotRV, alpha));
}

/**
 * Lighting via Phong illumination.
 *
 * The vec3 returned is the RGB color of that point after lighting is applied.
 * k_a: Ambient color
 * k_d: Diffuse color
 * k_s: Specular color
 * alpha: Shininess coefficient
 * p: position of point being lit
 * eye: the position of the camera
 *
 * See https://en.wikipedia.org/wiki/Phong_reflection_model#Description
 */
vec3 phongIllumination(vec3 k_a, vec3 k_d, vec3 k_s, float alpha, vec3 p, vec3 eye) {
	const vec3 ambientLight = 0.5 * vec3(1.0, 1.0, 1.0);
	vec3 color = ambientLight * k_a;

	vec3 light1Pos = vec3(4.0 * sin(time),
			2.0,
			4.0 * cos(time));
	vec3 light1Intensity = vec3(0.4, 0.4, 0.4);

	color += phongContribForLight(k_d, k_s, alpha, p, eye,
			light1Pos,
			light1Intensity);

	vec3 light2Pos = vec3(2.0 * sin(0.37 * time),
			2.0 * cos(0.37 * time),
			2.0);
	vec3 light2Intensity = vec3(0.4, 0.4, 0.4);

	color += phongContribForLight(k_d, k_s, alpha, p, eye,
			light2Pos,
			light2Intensity);
	return color;
}

void main() {
	vec3 dir = rayDirection(FOV, resolution.xy, gl_FragCoord.xy);
	float dist = shortestDistanceToSurface(camera_eye, dir, MIN_DIST, MAX_DIST);

	if (dist > MAX_DIST - EPSILON) {
		// Didn't hit anything
		FragColor = vec4(0.0, 0.0, 0.0, 0.0);
		return;
	}
	// The closest point on the surface to the eyepoint along the view ray
	vec3 p = camera_eye + dist * dir;

	vec3 K_a = vec3(0.2, 0.2, 0.2);
	vec3 K_d = vec3(0.7, 0.2, 0.2);
	vec3 K_s = vec3(1.0, 1.0, 1.0);
	vec3 color = phongIllumination(K_a, K_d, K_s, shininess, p, camera_eye);
	FragColor = vec4(color, 1.0);
}
